//! Removing an item, or lowering its quantity, by voice — and undoing that.
//!
//! Adding by voice goes through the confirmation queue
//! ([`crate::services::voice`]), because nothing reaches the list unasked.
//! Taking away is different: "Alexa, remove milk" can only shrink what the
//! household buys, so it changes the list at once. To keep it reversible,
//! each change is recorded with a snapshot of the item, and "Alexa, undo"
//! reverses the newest one ([`undo`]).
//!
//! Only `active` items change. A committed item is locked for purchase, as
//! it is for the web app, so a change to one is a 409 that tells the
//! household to release the list first.

mod log;
mod matching;
mod plan;
pub mod repository;
mod undo;

pub use undo::{undo_latest, UNDO_WINDOW};

use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::db::{GroceryItemStatus, IntakeSource, VoiceListChange};
use crate::models::schemas::AlexaRemoveCreate;
use crate::services::grocery::repository as items;
use crate::services::voice::Delivery;

/// Takes an item off the list, or lowers its quantity, as a voice channel
/// asked.
///
/// A request id seen before returns the change it made, as
/// [`Delivery::AlreadySeen`], without changing anything again.
///
/// Errors: [`ApiError::NotFound`] when no item on the list matches the name;
/// [`ApiError::Conflict`] when the matching item is committed for purchase.
pub async fn remove(
    pool: &PgPool,
    source: IntakeSource,
    payload: &AlexaRemoveCreate,
) -> Result<(VoiceListChange, Delivery), ApiError> {
    let mut tx = pool.begin().await?;
    repository::serialise(&mut tx).await?;

    let external_id = payload.external_id.as_deref();
    if let Some(external_id) = external_id {
        if let Some(change) = repository::find_by_external_id(&mut *tx, source, external_id).await?
        {
            tx.rollback().await?;
            log::redelivered(&change, external_id);
            return Ok((change, Delivery::AlreadySeen));
        }
    }

    let spoken = payload.item.trim();
    let variants = matching::name_variants(spoken);
    let Some(item) = repository::lock_item_named(&mut tx, &variants).await? else {
        tx.rollback().await?;
        log::no_match(spoken);
        return Err(ApiError::NotFound(format!("{spoken} is not on the list")));
    };
    if item.status != GroceryItemStatus::Active {
        tx.rollback().await?;
        return Err(ApiError::Conflict(format!(
            "{} is committed for purchase — release the list to change it",
            item.name
        )));
    }

    let quantity_after = plan::quantity_after(item.quantity, payload.quantity);
    if quantity_after == 0 {
        items::delete_item(&mut tx, item.id).await?;
    } else {
        items::set_item_quantity(&mut tx, item.id, quantity_after).await?;
    }
    let change =
        repository::insert_change(&mut tx, source, external_id, &item, quantity_after).await?;
    tx.commit().await?;
    log::applied(&change);
    Ok((change, Delivery::Recorded))
}
