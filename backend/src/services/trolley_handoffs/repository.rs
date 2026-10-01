//! Every SQL statement for `trolley_handoffs` and `trolley_handoff_lines`.
//!
//! The one place those tables are read or written. Every statement is a
//! literal `&'static str` with bind parameters.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{
    TrolleyHandoff, TrolleyHandoffLine, TrolleyHandoffStatus, TrolleyLineOutcome,
};
use crate::services::stores::Store;

/// A list item's chosen product at `store`, ready to become a handoff line:
/// `(grocery_item_id, product_id, product_name, quantity)`.
pub type ChosenLine = (Uuid, String, String, i32);

/// Every list item still to be bought (active or committed) whose chosen
/// product is at `store`, oldest item first. Quantities are the items' current
/// quantities, not the quantity the product was priced at.
pub async fn chosen_lines_for_store<'e, E>(
    executor: E,
    store: Store,
) -> Result<Vec<ChosenLine>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ChosenLine>(
        "SELECT i.id, s.product_id, s.product_name, i.quantity
         FROM grocery_items i
         JOIN item_selections s ON s.grocery_item_id = i.id
         WHERE s.store = $1 AND i.status IN ('active', 'committed')
         ORDER BY i.created_at, i.id",
    )
    .bind(store)
    .fetch_all(executor)
    .await?)
}

/// Marks every handoff for `store` that is still waiting as replaced, so only
/// the newest one can be claimed. Returns how many were replaced.
pub async fn replace_waiting<'e, E>(executor: E, store: Store) -> Result<u64, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query(
        "UPDATE trolley_handoffs SET status = 'replaced'
         WHERE store = $1 AND status = 'waiting_for_store_tab'",
    )
    .bind(store)
    .execute(executor)
    .await?
    .rows_affected())
}

/// Inserts a new waiting handoff.
pub async fn insert_handoff<'e, E>(
    executor: E,
    store: Store,
    created_by: &str,
    expires_at: DateTime<Utc>,
) -> Result<TrolleyHandoff, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(
        "INSERT INTO trolley_handoffs (id, store, status, created_by, expires_at)
         VALUES ($1, $2, 'waiting_for_store_tab', $3, $4)
         RETURNING id, store, status, created_by, created_at, expires_at, claimed_at, reported_at",
    )
    .bind(Uuid::new_v4())
    .bind(store)
    .bind(created_by)
    .bind(expires_at)
    .fetch_one(executor)
    .await?)
}

/// Inserts one line of a handoff; `position` keeps the list's order.
pub async fn insert_line<'e, E>(
    executor: E,
    handoff_id: Uuid,
    position: i32,
    line: &ChosenLine,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO trolley_handoff_lines
             (handoff_id, position, grocery_item_id, product_id, product_name, quantity)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(handoff_id)
    .bind(position)
    .bind(line.0)
    .bind(&line.1)
    .bind(&line.2)
    .bind(line.3)
    .execute(executor)
    .await?;
    Ok(())
}

/// The handoff with this id, if any.
pub async fn find_handoff<'e, E>(executor: E, id: Uuid) -> Result<Option<TrolleyHandoff>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(
        "SELECT id, store, status, created_by, created_at, expires_at, claimed_at, reported_at
         FROM trolley_handoffs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(executor)
    .await?)
}

/// The handoff with this id, locked for the rest of the transaction.
pub async fn lock_handoff<'e, E>(executor: E, id: Uuid) -> Result<Option<TrolleyHandoff>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(
        "SELECT id, store, status, created_by, created_at, expires_at, claimed_at, reported_at
         FROM trolley_handoffs WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(executor)
    .await?)
}

/// Claims the newest unexpired waiting handoff for `store`, if there is one.
/// Two tabs claiming at once cannot both get it (`SKIP LOCKED`).
pub async fn claim_newest_waiting<'e, E>(
    executor: E,
    store: Store,
) -> Result<Option<TrolleyHandoff>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(
        "UPDATE trolley_handoffs SET status = 'claimed_by_store_tab', claimed_at = now()
         WHERE id = (
             SELECT id FROM trolley_handoffs
             WHERE store = $1 AND status = 'waiting_for_store_tab' AND expires_at > now()
             ORDER BY created_at DESC
             LIMIT 1
             FOR UPDATE SKIP LOCKED
         )
         RETURNING id, store, status, created_by, created_at, expires_at, claimed_at, reported_at",
    )
    .bind(store)
    .fetch_optional(executor)
    .await?)
}

/// Every line of a handoff, in list order.
pub async fn lines_of<'e, E>(
    executor: E,
    handoff_id: Uuid,
) -> Result<Vec<TrolleyHandoffLine>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoffLine>(
        "SELECT handoff_id, grocery_item_id, product_id, product_name, quantity, outcome, problem
         FROM trolley_handoff_lines
         WHERE handoff_id = $1
         ORDER BY position",
    )
    .bind(handoff_id)
    .fetch_all(executor)
    .await?)
}

/// Writes the reported outcome onto every line of the handoff for `product_id`.
pub async fn set_outcome<'e, E>(
    executor: E,
    handoff_id: Uuid,
    product_id: &str,
    outcome: TrolleyLineOutcome,
    problem: Option<&str>,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE trolley_handoff_lines SET outcome = $3, problem = $4
         WHERE handoff_id = $1 AND product_id = $2",
    )
    .bind(handoff_id)
    .bind(product_id)
    .bind(outcome)
    .bind(problem)
    .execute(executor)
    .await?;
    Ok(())
}

/// Closes a claimed handoff with its final status.
pub async fn finish<'e, E>(
    executor: E,
    handoff_id: Uuid,
    status: TrolleyHandoffStatus,
) -> Result<TrolleyHandoff, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(
        "UPDATE trolley_handoffs SET status = $2, reported_at = now()
         WHERE id = $1
         RETURNING id, store, status, created_by, created_at, expires_at, claimed_at, reported_at",
    )
    .bind(handoff_id)
    .bind(status)
    .fetch_one(executor)
    .await?)
}
