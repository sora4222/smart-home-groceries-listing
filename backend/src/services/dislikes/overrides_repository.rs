//! Every SQL statement for `dislike_overrides`.
//!
//! The one place that table is read or written. Every statement is a literal
//! `&'static str` with bind parameters.

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::DislikeOverride;
use crate::services::stores::Store;

/// Every override on the list, oldest first.
pub async fn list_all<'e, E>(executor: E) -> Result<Vec<DislikeOverride>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, DislikeOverride>(
        "SELECT id, grocery_item_id, store, product_id, overridden_by, overridden_at
         FROM dislike_overrides
         ORDER BY overridden_at, id",
    )
    .fetch_all(executor)
    .await?)
}

/// The overrides on one list item.
pub async fn list_for_item<'e, E>(
    executor: E,
    grocery_item_id: Uuid,
) -> Result<Vec<DislikeOverride>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, DislikeOverride>(
        "SELECT id, grocery_item_id, store, product_id, overridden_by, overridden_at
         FROM dislike_overrides
         WHERE grocery_item_id = $1
         ORDER BY overridden_at, id",
    )
    .bind(grocery_item_id)
    .fetch_all(executor)
    .await?)
}

/// Sets a product's dislikes aside for one item. Doing it twice keeps the
/// first row, so `overridden_by` stays the member who asked first.
pub async fn upsert<'e, E>(
    executor: E,
    grocery_item_id: Uuid,
    store: Store,
    product_id: &str,
    overridden_by: &str,
) -> Result<DislikeOverride, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, DislikeOverride>(
        "INSERT INTO dislike_overrides (id, grocery_item_id, store, product_id, overridden_by)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (grocery_item_id, store, product_id) DO UPDATE SET
             product_id = EXCLUDED.product_id
         RETURNING id, grocery_item_id, store, product_id, overridden_by, overridden_at",
    )
    .bind(Uuid::new_v4())
    .bind(grocery_item_id)
    .bind(store)
    .bind(product_id)
    .bind(overridden_by)
    .fetch_one(executor)
    .await?)
}

/// Removes an override, so the dislikes count again. `false` means none.
pub async fn delete<'e, E>(
    executor: E,
    grocery_item_id: Uuid,
    store: Store,
    product_id: &str,
) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query(
        "DELETE FROM dislike_overrides
         WHERE grocery_item_id = $1 AND store = $2 AND product_id = $3",
    )
    .bind(grocery_item_id)
    .bind(store)
    .bind(product_id)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
