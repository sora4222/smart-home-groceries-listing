//! The confirmation queue's log events, one function per thing that happened.
//!
//! Kept out of the service so its methods read as rules, and so every event
//! carries the same field names: `request_id`, `source`, `item`.

use crate::models::db::{GroceryItem, VoiceRequest};

/// An intake channel delivered a new item, now waiting for a decision.
pub fn recorded(request: &VoiceRequest) {
    tracing::info!(
        request_id = %request.id,
        source = ?request.source,
        item = %request.parsed_name,
        quantity = request.parsed_quantity,
        "intake request recorded"
    );
}

/// A channel re-delivered an item already in the queue; nothing changed.
pub fn redelivered(request: &VoiceRequest, external_id: &str) {
    tracing::info!(
        external_id,
        request_id = %request.id,
        "ignoring re-delivered intake item"
    );
}

/// A request was accepted onto the list, as a new entry or merged into one.
pub fn accepted(request: &VoiceRequest, item: &GroceryItem, merged: bool, user_id: &str) {
    tracing::info!(
        request_id = %request.id,
        item_id = %item.id,
        item = %item.name,
        quantity = item.quantity,
        filter_terms = ?item.filter_terms,
        merged,
        user_id,
        "intake request accepted"
    );
}

/// Accepting would duplicate an active item and the household is being asked.
pub fn duplicate_asked(request: &VoiceRequest, existing: &GroceryItem) {
    tracing::info!(
        request_id = %request.id,
        existing_item_id = %existing.id,
        "intake request duplicates an active item; asking whether to merge"
    );
}

/// A request was rejected.
pub fn rejected(request: &VoiceRequest) {
    tracing::info!(
        request_id = %request.id,
        item = %request.parsed_name,
        "intake request rejected"
    );
}
