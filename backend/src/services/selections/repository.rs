//! Every SQL statement for `item_selections`.
//!
//! The one place that table is read or written, including the delete the
//! grocery list triggers when an item is renamed and the switch the order
//! planner makes between an item's choices at different stores. Every statement is a
//! literal `&'static str` with bind parameters.

use rust_decimal::Decimal;
use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::ItemSelection;
use crate::services::stores::{Basis, Store};

/// The columns a choice writes, taken from the store's search answer.
#[derive(Debug)]
pub struct NewSelection<'a> {
    pub grocery_item_id: Uuid,
    pub store: Store,
    pub product_id: &'a str,
    pub product_name: &'a str,
    pub brand: Option<&'a str>,
    pub package_size: Option<&'a str>,
    pub price: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub unit_price_per: Option<Basis>,
    pub total_price: Option<Decimal>,
    pub priced_quantity: i32,
    pub url: &'a str,
    pub selected_by: &'a str,
}

/// The choice each item's order buys (one per item), oldest first.
pub async fn list_for_order<'e, E>(executor: E) -> Result<Vec<ItemSelection>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemSelection>(
        "SELECT id, grocery_item_id, store, product_id, product_name, brand,
                package_size, price, unit_price, unit_price_per, total_price,
                priced_quantity, url, selected_by, selected_at, for_order
         FROM item_selections
         WHERE for_order
         ORDER BY selected_at",
    )
    .fetch_all(executor)
    .await?)
}

/// Every saved choice at every store, oldest first.
pub async fn list_every<'e, E>(executor: E) -> Result<Vec<ItemSelection>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemSelection>(
        "SELECT id, grocery_item_id, store, product_id, product_name, brand,
                package_size, price, unit_price, unit_price_per, total_price,
                priced_quantity, url, selected_by, selected_at, for_order
         FROM item_selections
         ORDER BY selected_at",
    )
    .fetch_all(executor)
    .await?)
}

/// Saves the item's choice at the new choice's store, replacing any earlier
/// one there, and makes it the choice the order buys. The item's choice at
/// another store is kept, no longer for the order.
pub async fn upsert(
    tx: &mut Transaction<'_, Postgres>,
    new: &NewSelection<'_>,
) -> Result<ItemSelection, ApiError> {
    // First, so the one-per-item `for_order` index never sees two.
    sqlx::query(
        "UPDATE item_selections SET for_order = false
         WHERE grocery_item_id = $1 AND store <> $2 AND for_order",
    )
    .bind(new.grocery_item_id)
    .bind(new.store)
    .execute(&mut **tx)
    .await?;
    Ok(sqlx::query_as::<_, ItemSelection>(
        "INSERT INTO item_selections
             (id, grocery_item_id, store, product_id, product_name, brand,
              package_size, price, unit_price, unit_price_per, total_price,
              priced_quantity, url, selected_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         ON CONFLICT (grocery_item_id, store) DO UPDATE SET
             product_id = EXCLUDED.product_id,
             product_name = EXCLUDED.product_name,
             brand = EXCLUDED.brand,
             package_size = EXCLUDED.package_size,
             price = EXCLUDED.price,
             unit_price = EXCLUDED.unit_price,
             unit_price_per = EXCLUDED.unit_price_per,
             total_price = EXCLUDED.total_price,
             priced_quantity = EXCLUDED.priced_quantity,
             url = EXCLUDED.url,
             selected_by = EXCLUDED.selected_by,
             selected_at = now(),
             for_order = true
         RETURNING id, grocery_item_id, store, product_id, product_name, brand,
                   package_size, price, unit_price, unit_price_per, total_price,
                   priced_quantity, url, selected_by, selected_at, for_order",
    )
    .bind(Uuid::new_v4())
    .bind(new.grocery_item_id)
    .bind(new.store)
    .bind(new.product_id)
    .bind(new.product_name)
    .bind(new.brand)
    .bind(new.package_size)
    .bind(new.price)
    .bind(new.unit_price)
    .bind(new.unit_price_per)
    .bind(new.total_price)
    .bind(new.priced_quantity)
    .bind(new.url)
    .bind(new.selected_by)
    .fetch_one(&mut **tx)
    .await?)
}

/// Forgets an item's choices at every store. `false` means it had none.
pub async fn delete_for_item<'e, E>(executor: E, grocery_item_id: Uuid) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query("DELETE FROM item_selections WHERE grocery_item_id = $1")
        .bind(grocery_item_id)
        .execute(executor)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Forgets the item's choice at `store`. If that was the one the order
/// bought, the item's other choice (if any) becomes it. `false` means there
/// was no choice at `store`.
pub async fn delete_at_store(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
    store: Store,
) -> Result<bool, ApiError> {
    let result =
        sqlx::query("DELETE FROM item_selections WHERE grocery_item_id = $1 AND store = $2")
            .bind(grocery_item_id)
            .bind(store)
            .execute(&mut **tx)
            .await?;
    sqlx::query(
        "UPDATE item_selections SET for_order = true
         WHERE grocery_item_id = $1
           AND NOT EXISTS (SELECT 1 FROM item_selections
                           WHERE grocery_item_id = $1 AND for_order)",
    )
    .bind(grocery_item_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Makes the item's choice at `store` the one the order buys. `false` means
/// the item has no choice at `store`, and nothing changed.
pub async fn buy_at_store(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
    store: Store,
) -> Result<bool, ApiError> {
    let has_choice: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM item_selections
                        WHERE grocery_item_id = $1 AND store = $2)",
    )
    .bind(grocery_item_id)
    .bind(store)
    .fetch_one(&mut **tx)
    .await?;
    if !has_choice {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE item_selections SET for_order = false
         WHERE grocery_item_id = $1 AND store <> $2 AND for_order",
    )
    .bind(grocery_item_id)
    .bind(store)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "UPDATE item_selections SET for_order = true
         WHERE grocery_item_id = $1 AND store = $2",
    )
    .bind(grocery_item_id)
    .bind(store)
    .execute(&mut **tx)
    .await?;
    Ok(true)
}
