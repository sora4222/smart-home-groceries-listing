//! Every SQL statement for `item_selections`.
//!
//! The one place that table is read or written, including the delete the
//! grocery list triggers when an item is renamed, the switch the order
//! planner makes between an item's stores, and the snapshots Undo puts
//! back. Every statement is a literal `&'static str` with bind parameters.

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
    pub image_url: Option<&'a str>,
    pub selected_by: &'a str,
    /// How many to buy; `None` for the only product at its store.
    pub quantity: Option<i32>,
}

/// The choice each item's order buys (one per item), oldest first.
pub async fn list_for_order<'e, E>(executor: E) -> Result<Vec<ItemSelection>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemSelection>(
        "SELECT id, grocery_item_id, store, product_id, product_name, brand,
                package_size, price, unit_price, unit_price_per, total_price,
                priced_quantity, url, image_url, selected_by, selected_at, for_order,
                quantity
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
                priced_quantity, url, image_url, selected_by, selected_at, for_order,
                quantity
         FROM item_selections
         ORDER BY selected_at",
    )
    .fetch_all(executor)
    .await?)
}

/// Saves the product as one of the item's choices at its store (refreshing
/// it when already chosen, keeping its count), and makes that store's
/// products the ones the order buys. Choices at other stores are kept, no
/// longer for the order.
pub async fn upsert(
    tx: &mut Transaction<'_, Postgres>,
    new: &NewSelection<'_>,
) -> Result<ItemSelection, ApiError> {
    sqlx::query(
        "UPDATE item_selections SET for_order = (store = $2)
         WHERE grocery_item_id = $1",
    )
    .bind(new.grocery_item_id)
    .bind(new.store)
    .execute(&mut **tx)
    .await?;
    Ok(sqlx::query_as::<_, ItemSelection>(
        "INSERT INTO item_selections
             (id, grocery_item_id, store, product_id, product_name, brand,
              package_size, price, unit_price, unit_price_per, total_price,
              priced_quantity, url, image_url, selected_by, quantity)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
         ON CONFLICT (grocery_item_id, store, product_id) DO UPDATE SET
             product_name = EXCLUDED.product_name,
             brand = EXCLUDED.brand,
             package_size = EXCLUDED.package_size,
             price = EXCLUDED.price,
             unit_price = EXCLUDED.unit_price,
             unit_price_per = EXCLUDED.unit_price_per,
             total_price = EXCLUDED.total_price,
             priced_quantity = EXCLUDED.priced_quantity,
             url = EXCLUDED.url,
             image_url = EXCLUDED.image_url,
             selected_by = EXCLUDED.selected_by,
             selected_at = now(),
             for_order = true
         RETURNING id, grocery_item_id, store, product_id, product_name, brand,
                   package_size, price, unit_price, unit_price_per, total_price,
                   priced_quantity, url, image_url, selected_by, selected_at, for_order,
                   quantity",
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
    .bind(new.image_url)
    .bind(new.selected_by)
    .bind(new.quantity)
    .fetch_one(&mut **tx)
    .await?)
}

/// The item's products at `store`, oldest choice first, locked.
pub async fn lock_at_store(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
    store: Store,
) -> Result<Vec<ItemSelection>, ApiError> {
    Ok(sqlx::query_as::<_, ItemSelection>(
        "SELECT id, grocery_item_id, store, product_id, product_name, brand,
                package_size, price, unit_price, unit_price_per, total_price,
                priced_quantity, url, image_url, selected_by, selected_at, for_order,
                quantity
         FROM item_selections
         WHERE grocery_item_id = $1 AND store = $2
         ORDER BY selected_at, id
         FOR UPDATE",
    )
    .bind(grocery_item_id)
    .bind(store)
    .fetch_all(&mut **tx)
    .await?)
}

/// Stores how many of one chosen product to buy.
pub async fn set_quantity(
    tx: &mut Transaction<'_, Postgres>,
    selection_id: Uuid,
    quantity: i32,
) -> Result<(), ApiError> {
    sqlx::query("UPDATE item_selections SET quantity = $2 WHERE id = $1")
        .bind(selection_id)
        .bind(quantity)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Forgets every product at `store` except `keep`: choosing for an item that
/// wants one replaces its product.
pub async fn delete_others_at_store(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
    store: Store,
    keep: &str,
) -> Result<(), ApiError> {
    sqlx::query(
        "DELETE FROM item_selections
         WHERE grocery_item_id = $1 AND store = $2 AND product_id <> $3",
    )
    .bind(grocery_item_id)
    .bind(store)
    .bind(keep)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Forgets one product at `store`. `false` means it was not chosen.
pub async fn delete_product(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
    store: Store,
    product_id: &str,
) -> Result<bool, ApiError> {
    let result = sqlx::query(
        "DELETE FROM item_selections
         WHERE grocery_item_id = $1 AND store = $2 AND product_id = $3",
    )
    .bind(grocery_item_id)
    .bind(store)
    .bind(product_id)
    .execute(&mut **tx)
    .await?;
    hand_over_for_order(tx, grocery_item_id).await?;
    Ok(result.rows_affected() > 0)
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

/// Forgets the item's products at `store`. If the order bought from there,
/// the item's other store (if any) becomes it. `false` means there was no
/// choice at `store`.
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
    hand_over_for_order(tx, grocery_item_id).await?;
    Ok(result.rows_affected() > 0)
}

/// When nothing left is for the order, the remaining products (all at one
/// store, as there are two) become it.
async fn hand_over_for_order(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
) -> Result<(), ApiError> {
    sqlx::query(
        "UPDATE item_selections SET for_order = true
         WHERE grocery_item_id = $1
           AND NOT EXISTS (SELECT 1 FROM item_selections
                           WHERE grocery_item_id = $1 AND for_order)",
    )
    .bind(grocery_item_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
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

/// Saves the item's products as a new snapshot for Undo, and forgets
/// snapshots older than a day. Returns the snapshot's id.
pub async fn save_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    grocery_item_id: Uuid,
) -> Result<Uuid, ApiError> {
    sqlx::query("DELETE FROM selection_snapshots WHERE taken_at < now() - interval '1 day'")
        .execute(&mut **tx)
        .await?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO selection_snapshots (id, grocery_item_id) VALUES ($1, $2)")
        .bind(id)
        .bind(grocery_item_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "INSERT INTO selection_snapshot_rows
             (snapshot_id, id, grocery_item_id, store, product_id, product_name, brand,
              package_size, price, unit_price, unit_price_per, total_price,
              priced_quantity, url, image_url, selected_by, selected_at, for_order,
              quantity)
         SELECT $1, id, grocery_item_id, store, product_id, product_name, brand,
                package_size, price, unit_price, unit_price_per, total_price,
                priced_quantity, url, image_url, selected_by, selected_at, for_order,
                quantity
         FROM item_selections
         WHERE grocery_item_id = $2",
    )
    .bind(id)
    .bind(grocery_item_id)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

/// The item a snapshot was taken of, with the snapshot locked. `None` when
/// there is no such snapshot (or it was older than a day).
pub async fn lock_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    snapshot_id: Uuid,
) -> Result<Option<Uuid>, ApiError> {
    Ok(sqlx::query_scalar(
        "SELECT grocery_item_id FROM selection_snapshots WHERE id = $1 FOR UPDATE",
    )
    .bind(snapshot_id)
    .fetch_optional(&mut **tx)
    .await?)
}

/// Replaces the item's products with the snapshot's, each as it was saved.
/// New row ids, so a snapshot can be put back more than once.
pub async fn put_back_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    snapshot_id: Uuid,
    grocery_item_id: Uuid,
) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM item_selections WHERE grocery_item_id = $1")
        .bind(grocery_item_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "INSERT INTO item_selections
             (id, grocery_item_id, store, product_id, product_name, brand,
              package_size, price, unit_price, unit_price_per, total_price,
              priced_quantity, url, image_url, selected_by, selected_at, for_order,
              quantity)
         SELECT gen_random_uuid(), $2, store, product_id, product_name, brand,
                package_size, price, unit_price, unit_price_per, total_price,
                priced_quantity, url, image_url, selected_by, selected_at, for_order,
                quantity
         FROM selection_snapshot_rows
         WHERE snapshot_id = $1",
    )
    .bind(snapshot_id)
    .bind(grocery_item_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
