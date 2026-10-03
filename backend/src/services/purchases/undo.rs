//! Undo: forgetting a shop that was saved as bought by mistake.
//!
//! The order and its purchases are deleted, and each list item they
//! fulfilled goes back to the status it had (active or committed) — if it is
//! still `ordered`. An item deleted since stays deleted.

use sqlx::PgPool;
use uuid::Uuid;

use super::{log, repository};
use crate::error::ApiError;
use crate::services::grocery::repository as items;

/// What Undo changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Undone {
    /// List items put back on the list.
    pub items_restored: usize,
}

/// Deletes order `id` and puts its items back. 404 when there is no such
/// order (already undone in another tab, for example).
pub async fn undo(pool: &PgPool, id: Uuid, user_id: &str) -> Result<Undone, ApiError> {
    let mut tx = pool.begin().await?;
    let Some(order) = repository::lock_order(&mut *tx, id).await? else {
        tx.rollback().await?;
        return Err(ApiError::NotFound(
            "This shop is not saved as bought (it may be undone already).".to_string(),
        ));
    };
    let mut items_restored = 0;
    for purchase in repository::purchases_of(&mut *tx, id).await? {
        let Some(item_id) = purchase.grocery_item_id else {
            continue;
        };
        if items::unmark_ordered(&mut *tx, item_id, purchase.item_status_before).await? {
            items_restored += 1;
        }
    }
    repository::delete_order(&mut *tx, id).await?;
    tx.commit().await?;
    log::undone(&order, items_restored, user_id);
    Ok(Undone { items_restored })
}
