//! The reads of `purchases` that the product picker and the spending
//! analysis need. Writes are in [`super::repository`].
//!
//! Optional filters are bound as `NULL` and tested in the statement
//! (`$1::timestamptz IS NULL OR ...`), so every statement stays a literal.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

use super::repository::purchase_columns;
use crate::error::ApiError;
use crate::models::db::Purchase;
use crate::services::stores::Store;

/// What the spending analysis narrows purchases to. `None` = no limit.
#[derive(Debug, Default)]
pub struct PurchaseFilter<'a> {
    /// Bought at or after this instant.
    pub from: Option<DateTime<Utc>>,
    /// Bought before this instant.
    pub until: Option<DateTime<Utc>>,
    pub store: Option<Store>,
    /// Part of the item's key (lowercase, spaces collapsed).
    pub item_key_part: Option<&'a str>,
    pub category: Option<&'a str>,
}

/// Every purchase of the given products, newest first. `products` pairs a
/// store with its own product id.
pub async fn of_products<'e, E>(
    executor: E,
    products: &[(Store, String)],
) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    let stores: Vec<&str> = products.iter().map(|(store, _)| store.as_str()).collect();
    let ids: Vec<&str> = products.iter().map(|(_, id)| id.as_str()).collect();
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        " FROM purchases
         WHERE (store, product_id) IN (SELECT * FROM unnest($1::text[], $2::text[]))
         ORDER BY bought_at DESC, id"
    ))
    .bind(&stores)
    .bind(&ids)
    .fetch_all(executor)
    .await?)
}

/// Every purchase the filter lets through, oldest first.
pub async fn matching<'e, E>(
    executor: E,
    filter: &PurchaseFilter<'_>,
) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        " FROM purchases
         WHERE ($1::timestamptz IS NULL OR bought_at >= $1)
           AND ($2::timestamptz IS NULL OR bought_at < $2)
           AND ($3::text IS NULL OR store = $3)
           AND ($4::text IS NULL OR strpos(item_key, $4) > 0)
           AND ($5::text IS NULL OR category = $5)
         ORDER BY bought_at, id"
    ))
    .bind(filter.from)
    .bind(filter.until)
    .bind(filter.store.map(Store::as_str))
    .bind(filter.item_key_part)
    .bind(filter.category)
    .fetch_all(executor)
    .await?)
}

/// Every purchase for one item, oldest first.
pub async fn of_item<'e, E>(executor: E, item_key: &str) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        " FROM purchases WHERE item_key = $1 ORDER BY bought_at, id"
    ))
    .bind(item_key)
    .fetch_all(executor)
    .await?)
}

/// Every category any purchase is in, A to Z.
pub async fn categories<'e, E>(executor: E) -> Result<Vec<String>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(
        sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT category FROM purchases ORDER BY category",
        )
        .fetch_all(executor)
        .await?,
    )
}
