//! Every SQL statement for `trolley_handoff_lines`, and the read of the list
//! items a new handoff is made from.
//!
//! Every statement is a literal `&'static str` with bind parameters.

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{TrolleyHandoffLine, TrolleyLineOutcome};
use crate::services::stores::Store;

/// A list item's chosen product at `store`, ready to become a handoff line:
/// `(grocery_item_id, product_id, product_name, quantity)`.
pub type ChosenLine = (Uuid, String, String, i32);

/// Every list item still to be bought (active or committed) whose order buys
/// its chosen product at `store`, oldest item first. An item's choice at
/// another store, not the one to buy, is never sent. Quantities are the items' current
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
         WHERE s.store = $1 AND s.for_order AND i.status IN ('active', 'committed')
         ORDER BY i.created_at, i.id",
    )
    .bind(store)
    .fetch_all(executor)
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
