//! Log events for voice list changes, one function per thing that happened.
//!
//! Every event carries the same field names: `change_id`, `item_id`, `item`.

use crate::models::db::VoiceListChange;

/// A spoken remove or reduce changed the list.
pub fn applied(change: &VoiceListChange) {
    tracing::info!(
        change_id = %change.id,
        item_id = %change.grocery_item_id,
        item = %change.item_name,
        kind = ?change.kind,
        quantity_before = change.quantity_before,
        quantity_after = change.quantity_after,
        "voice list change applied"
    );
}

/// The channel re-delivered a request already handled; nothing changed.
pub fn redelivered(change: &VoiceListChange, external_id: &str) {
    tracing::info!(
        external_id,
        change_id = %change.id,
        "ignoring re-delivered voice list request"
    );
}

/// No item on the list matched the spoken name.
pub fn no_match(spoken: &str) {
    tracing::info!(item = %spoken, "voice list change matched no item");
}

/// Undo reversed a change.
pub fn undone(change: &VoiceListChange) {
    tracing::info!(
        change_id = %change.id,
        item_id = %change.grocery_item_id,
        item = %change.item_name,
        kind = ?change.kind,
        "voice list change undone"
    );
}
