//! Every SQL statement for `trolley_handoffs` (lines: [`super::line_repository`]).
//!
//! Every statement is a literal `&'static str` with bind parameters. The
//! column list is written once, in [`handoff_columns!`], and spliced in with
//! `concat!` at compile time, so the row type and every query stay in step.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{TrolleyHandoff, TrolleyHandoffStatus};
use crate::services::stores::Store;

use super::delivery::{DeliveryRequest, ReportedDelivery};

/// The columns [`TrolleyHandoff`] reads, as a string literal.
macro_rules! handoff_columns {
    () => {
        "id, store, status, created_by, created_at, expires_at, claimed_at, reported_at,
         delivery_date, delivery_time_of_day, delivery_outcome, delivery_window_label,
         delivery_window_start, delivery_window_end, delivery_fee, delivery_problem"
    };
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

/// Inserts a new waiting handoff with the delivery time asked for.
pub async fn insert_handoff<'e, E>(
    executor: E,
    store: Store,
    created_by: &str,
    expires_at: DateTime<Utc>,
    delivery: &DeliveryRequest,
) -> Result<TrolleyHandoff, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(concat!(
        "INSERT INTO trolley_handoffs
             (id, store, status, created_by, expires_at, delivery_date, delivery_time_of_day)
         VALUES ($1, $2, 'waiting_for_store_tab', $3, $4, $5, $6)
         RETURNING ",
        handoff_columns!()
    ))
    .bind(Uuid::new_v4())
    .bind(store)
    .bind(created_by)
    .bind(expires_at)
    .bind(delivery.date)
    .bind(delivery.time_of_day)
    .fetch_one(executor)
    .await?)
}

/// The handoff with this id, if any.
pub async fn find_handoff<'e, E>(executor: E, id: Uuid) -> Result<Option<TrolleyHandoff>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(concat!(
        "SELECT ",
        handoff_columns!(),
        " FROM trolley_handoffs WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await?)
}

/// The handoff with this id, locked for the rest of the transaction.
pub async fn lock_handoff<'e, E>(executor: E, id: Uuid) -> Result<Option<TrolleyHandoff>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(concat!(
        "SELECT ",
        handoff_columns!(),
        " FROM trolley_handoffs WHERE id = $1 FOR UPDATE"
    ))
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
    Ok(sqlx::query_as::<_, TrolleyHandoff>(concat!(
        "UPDATE trolley_handoffs SET status = 'claimed_by_store_tab', claimed_at = now()
         WHERE id = (
             SELECT id FROM trolley_handoffs
             WHERE store = $1 AND status = 'waiting_for_store_tab' AND expires_at > now()
             ORDER BY created_at DESC
             LIMIT 1
             FOR UPDATE SKIP LOCKED
         )
         RETURNING ",
        handoff_columns!()
    ))
    .bind(store)
    .fetch_optional(executor)
    .await?)
}

/// Closes a claimed handoff with its final status and what the store tab did
/// about the delivery window (all delivery columns `NULL` when it said nothing).
pub async fn finish<'e, E>(
    executor: E,
    handoff_id: Uuid,
    status: TrolleyHandoffStatus,
    delivery: Option<&ReportedDelivery>,
) -> Result<TrolleyHandoff, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, TrolleyHandoff>(concat!(
        "UPDATE trolley_handoffs SET status = $2, reported_at = now(),
             delivery_outcome = $3, delivery_window_label = $4, delivery_window_start = $5,
             delivery_window_end = $6, delivery_fee = $7, delivery_problem = $8
         WHERE id = $1
         RETURNING ",
        handoff_columns!()
    ))
    .bind(handoff_id)
    .bind(status)
    .bind(delivery.map(|d| d.outcome))
    .bind(delivery.and_then(|d| d.window_label.as_deref()))
    .bind(delivery.and_then(|d| d.window_start))
    .bind(delivery.and_then(|d| d.window_end))
    .bind(delivery.and_then(|d| d.fee))
    .bind(delivery.and_then(|d| d.problem.as_deref()))
    .fetch_one(executor)
    .await?)
}
