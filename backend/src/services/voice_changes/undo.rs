//! Undo: reversing the newest voice change, by voice.
//!
//! A removed item is put back as it was (same id, quantity, note and chips).
//! A reduced item gets the amount taken off added back to whatever it holds
//! now, so an edit made in the web app since is kept.

use chrono::{Duration, Utc};
use sqlx::PgPool;

use super::{log, repository};
use crate::error::ApiError;
use crate::models::db::{GroceryItemStatus, IntakeSource, VoiceChangeKind, VoiceListChange};
use crate::models::schemas::MAX_QUANTITY;
use crate::services::grocery::repository as items;
use crate::services::voice::Delivery;

/// How far back "undo" reaches. Older changes are settled; the web app is the
/// place to put such an item back.
pub const UNDO_WINDOW: Duration = Duration::minutes(30);

/// Reverses the newest change from `source` made within [`UNDO_WINDOW`] that
/// still stands. Saying "undo" again reverses the one before it.
///
/// A request id seen before returns the change it reversed, as
/// [`Delivery::AlreadySeen`], so a retried Undo never reverses a second one.
///
/// Errors: [`ApiError::NotFound`] when there is nothing to undo;
/// [`ApiError::Conflict`] when a reduced item has since been deleted or
/// committed for purchase, so its quantity cannot be put back.
pub async fn undo_latest(
    pool: &PgPool,
    source: IntakeSource,
    external_id: Option<&str>,
) -> Result<(VoiceListChange, Delivery), ApiError> {
    let mut tx = pool.begin().await?;
    repository::serialise(&mut tx).await?;

    if let Some(external_id) = external_id {
        let seen = repository::find_by_undo_external_id(&mut *tx, source, external_id).await?;
        if let Some(change) = seen {
            tx.rollback().await?;
            log::redelivered(&change, external_id);
            return Ok((change, Delivery::AlreadySeen));
        }
    }

    let since = Utc::now() - UNDO_WINDOW;
    let Some(change) = repository::lock_newest_standing(&mut tx, source, since).await? else {
        tx.rollback().await?;
        return Err(ApiError::NotFound(
            "there is no recent voice change to undo".to_string(),
        ));
    };

    match change.kind {
        VoiceChangeKind::Removed => {
            repository::restore_item(&mut tx, &change).await?;
        }
        VoiceChangeKind::Reduced => {
            let current = items::lock_item(&mut tx, change.grocery_item_id).await?;
            let Some(item) = current.filter(|item| item.status == GroceryItemStatus::Active) else {
                tx.rollback().await?;
                return Err(ApiError::Conflict(format!(
                    "{} has been deleted or committed since, so it cannot be changed back",
                    change.item_name
                )));
            };
            let taken_off = change.quantity_before - change.quantity_after;
            let restored = item.quantity.saturating_add(taken_off).min(MAX_QUANTITY);
            items::set_item_quantity(&mut tx, item.id, restored).await?;
        }
    }

    let change = repository::mark_undone(&mut tx, change.id, external_id).await?;
    tx.commit().await?;
    log::undone(&change);
    Ok((change, Delivery::Recorded))
}
