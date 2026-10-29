//! Making room at a store for a newly chosen product.
//!
//! An item wanting one keeps one product per store: a new choice replaces
//! the old. An item wanting more may keep up to that many products at a
//! store; the new one buys one, taken from the product buying the most.

use sqlx::{Postgres, Transaction};

use super::allocation;
use super::repository;
use crate::error::ApiError;
use crate::models::db::GroceryItem;
use crate::services::stores::Store;

/// Gets `store` ready for `product_id` on `item`, and returns the count the
/// product should be saved with (`None` when it is or stays the only one).
/// Refused with 409 when the item already has as many products there as it
/// wants.
pub async fn make_room(
    tx: &mut Transaction<'_, Postgres>,
    item: &GroceryItem,
    store: Store,
    product_id: &str,
) -> Result<Option<i32>, ApiError> {
    let at_store = repository::lock_at_store(tx, item.id, store).await?;
    if let Some(chosen) = at_store.iter().find(|row| row.product_id == product_id) {
        // Chosen already: keep its count.
        return Ok(chosen.quantity);
    }
    if at_store.is_empty() || item.quantity <= 1 {
        repository::delete_others_at_store(tx, item.id, store, product_id).await?;
        return Ok(None);
    }

    let stored: Vec<_> = at_store.iter().map(|row| row.quantity).collect();
    let counts = allocation::allocate(&stored, item.quantity);
    let Some(counts) = allocation::make_room(&counts) else {
        return Err(ApiError::Conflict(format!(
            "{} already has {} products at {}. Clear one, or raise the quantity.",
            item.name,
            at_store.len(),
            store.display_name()
        )));
    };
    for (row, count) in at_store.iter().zip(counts) {
        repository::set_quantity(tx, row.id, count).await?;
    }
    Ok(Some(1))
}
