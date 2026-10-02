//! Every SQL statement that writes `purchase_orders` and `purchases`, and the
//! reads Undo and the order list need. The analysis reads are in
//! [`super::read_repository`].
//!
//! Every statement is a literal `&'static str` with bind parameters.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{CategorySource, GroceryItemStatus, Purchase, PurchaseOrder};
use crate::services::stores::Store;

/// The `purchase_orders` columns, in [`PurchaseOrder`]'s order.
macro_rules! order_columns {
    () => {
        "id, store, items_total, delivery_fee, recorded_by, source, trolley_handoff_id, bought_at"
    };
}

/// The `purchases` columns, in [`Purchase`]'s order.
macro_rules! purchase_columns {
    () => {
        "id, order_id, grocery_item_id, item_name, item_key, item_status_before, store,
         product_id, product_name, brand, package_size, quantity, unit_price, shelf_price,
         total_price, delivery_fee_share, store_category, category, category_source, bought_at"
    };
}
pub(super) use purchase_columns;

/// The columns an order from a trolley fill writes.
#[derive(Debug)]
pub struct NewOrder<'a> {
    pub store: Store,
    pub items_total: Decimal,
    pub delivery_fee: Decimal,
    pub recorded_by: &'a str,
    pub trolley_handoff_id: Uuid,
    pub bought_at: DateTime<Utc>,
}

/// The columns one purchase writes.
#[derive(Debug)]
pub struct NewPurchase<'a> {
    pub grocery_item_id: Uuid,
    pub item_name: &'a str,
    pub item_key: &'a str,
    pub item_status_before: GroceryItemStatus,
    pub store: Store,
    pub product_id: &'a str,
    pub product_name: &'a str,
    pub brand: Option<&'a str>,
    pub package_size: Option<&'a str>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub shelf_price: Option<Decimal>,
    pub total_price: Decimal,
    pub delivery_fee_share: Decimal,
    pub store_category: Option<&'a str>,
    pub category: &'a str,
    pub category_source: CategorySource,
}

/// Saves an order from a trolley fill. `None` when that handoff was already
/// saved — one report is never counted twice.
pub async fn insert_order<'e, E>(
    executor: E,
    new: &NewOrder<'_>,
) -> Result<Option<PurchaseOrder>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, PurchaseOrder>(concat!(
        "INSERT INTO purchase_orders
             (id, store, items_total, delivery_fee, recorded_by, source,
              trolley_handoff_id, bought_at)
         VALUES ($1, $2, $3, $4, $5, 'trolley_fill', $6, $7)
         ON CONFLICT (trolley_handoff_id) DO NOTHING
         RETURNING ",
        order_columns!()
    ))
    .bind(Uuid::new_v4())
    .bind(new.store)
    .bind(new.items_total)
    .bind(new.delivery_fee)
    .bind(new.recorded_by)
    .bind(new.trolley_handoff_id)
    .bind(new.bought_at)
    .fetch_optional(executor)
    .await?)
}

/// Saves one purchase of an order.
pub async fn insert_purchase<'e, E>(
    executor: E,
    order: &PurchaseOrder,
    new: &NewPurchase<'_>,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO purchases
             (id, order_id, grocery_item_id, item_name, item_key, item_status_before, store,
              product_id, product_name, brand, package_size, quantity, unit_price, shelf_price,
              total_price, delivery_fee_share, store_category, category, category_source,
              bought_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                 $17, $18, $19, $20)",
    )
    .bind(Uuid::new_v4())
    .bind(order.id)
    .bind(new.grocery_item_id)
    .bind(new.item_name)
    .bind(new.item_key)
    .bind(new.item_status_before)
    .bind(new.store)
    .bind(new.product_id)
    .bind(new.product_name)
    .bind(new.brand)
    .bind(new.package_size)
    .bind(new.quantity)
    .bind(new.unit_price)
    .bind(new.shelf_price)
    .bind(new.total_price)
    .bind(new.delivery_fee_share)
    .bind(new.store_category)
    .bind(new.category)
    .bind(new.category_source)
    .bind(order.bought_at)
    .execute(executor)
    .await?;
    Ok(())
}

/// The newest orders first, at most `limit`.
pub async fn recent_orders<'e, E>(executor: E, limit: i64) -> Result<Vec<PurchaseOrder>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, PurchaseOrder>(concat!(
        "SELECT ",
        order_columns!(),
        " FROM purchase_orders ORDER BY bought_at DESC, id LIMIT $1"
    ))
    .bind(limit)
    .fetch_all(executor)
    .await?)
}

/// How many purchases each of `order_ids` holds: `(order_id, count)`.
pub async fn line_counts<'e, E>(
    executor: E,
    order_ids: &[Uuid],
) -> Result<Vec<(Uuid, i64)>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, (Uuid, i64)>(
        "SELECT order_id, count(*) FROM purchases
         WHERE order_id = ANY($1)
         GROUP BY order_id",
    )
    .bind(order_ids)
    .fetch_all(executor)
    .await?)
}

/// Locks one order for the rest of the transaction.
pub async fn lock_order<'e, E>(executor: E, id: Uuid) -> Result<Option<PurchaseOrder>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, PurchaseOrder>(concat!(
        "SELECT ",
        order_columns!(),
        " FROM purchase_orders WHERE id = $1 FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await?)
}

/// Every purchase of one order.
pub async fn purchases_of<'e, E>(executor: E, order_id: Uuid) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        " FROM purchases WHERE order_id = $1 ORDER BY item_name"
    ))
    .bind(order_id)
    .fetch_all(executor)
    .await?)
}

/// Deletes an order; its purchases go with it.
pub async fn delete_order<'e, E>(executor: E, id: Uuid) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM purchase_orders WHERE id = $1")
        .bind(id)
        .execute(executor)
        .await?;
    Ok(())
}

/// Every purchase's id and the texts its category is worked out from:
/// `(id, store_category, product_name)`.
pub async fn category_inputs<'e, E>(
    executor: E,
) -> Result<Vec<(Uuid, Option<String>, String)>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, (Uuid, Option<String>, String)>(
        "SELECT id, store_category, product_name FROM purchases",
    )
    .fetch_all(executor)
    .await?)
}

/// Sets one purchase's category; `true` when it changed.
pub async fn set_category<'e, E>(
    executor: E,
    id: Uuid,
    category: &str,
    source: CategorySource,
) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query(
        "UPDATE purchases SET category = $2, category_source = $3
         WHERE id = $1 AND (category <> $2 OR category_source <> $3)",
    )
    .bind(id)
    .bind(category)
    .bind(source)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
