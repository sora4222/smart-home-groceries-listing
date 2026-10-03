//! Every SQL statement for `product_dislikes`.
//!
//! The one place that table is read or written. Every statement is a literal
//! `&'static str` with bind parameters.

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::ProductDislike;
use crate::services::stores::Store;

/// The columns a dislike writes.
#[derive(Debug)]
pub struct NewDislike<'a> {
    pub user_id: &'a str,
    pub user_name: &'a str,
    pub store: Store,
    pub product_id: &'a str,
    pub product_name: &'a str,
    pub brand: Option<&'a str>,
    pub package_size: Option<&'a str>,
}

/// Every member's dislikes, newest first: the household view.
pub async fn list_all<'e, E>(executor: E) -> Result<Vec<ProductDislike>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ProductDislike>(
        "SELECT id, user_id, user_name, store, product_id, product_name, brand,
                package_size, disliked_at
         FROM product_dislikes
         ORDER BY disliked_at DESC, id",
    )
    .fetch_all(executor)
    .await?)
}

/// Every member's dislike of one product.
pub async fn list_for_product<'e, E>(
    executor: E,
    store: Store,
    product_id: &str,
) -> Result<Vec<ProductDislike>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ProductDislike>(
        "SELECT id, user_id, user_name, store, product_id, product_name, brand,
                package_size, disliked_at
         FROM product_dislikes
         WHERE store = $1 AND product_id = $2
         ORDER BY disliked_at, id",
    )
    .bind(store)
    .bind(product_id)
    .fetch_all(executor)
    .await?)
}

/// Saves a member's dislike. Disliking again keeps the first date and only
/// refreshes the labels, so the row says when the member first said so.
pub async fn upsert<'e, E>(executor: E, new: &NewDislike<'_>) -> Result<ProductDislike, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ProductDislike>(
        "INSERT INTO product_dislikes
             (id, user_id, user_name, store, product_id, product_name, brand, package_size)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (user_id, store, product_id) DO UPDATE SET
             user_name = EXCLUDED.user_name,
             product_name = EXCLUDED.product_name,
             brand = EXCLUDED.brand,
             package_size = EXCLUDED.package_size
         RETURNING id, user_id, user_name, store, product_id, product_name, brand,
                   package_size, disliked_at",
    )
    .bind(Uuid::new_v4())
    .bind(new.user_id)
    .bind(new.user_name)
    .bind(new.store)
    .bind(new.product_id)
    .bind(new.product_name)
    .bind(new.brand)
    .bind(new.package_size)
    .fetch_one(executor)
    .await?)
}

/// Removes one member's dislike. `false` means they had none.
pub async fn delete_for_user<'e, E>(
    executor: E,
    user_id: &str,
    store: Store,
    product_id: &str,
) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query(
        "DELETE FROM product_dislikes
         WHERE user_id = $1 AND store = $2 AND product_id = $3",
    )
    .bind(user_id)
    .bind(store)
    .bind(product_id)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
