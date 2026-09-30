//! The grocery list's log events, one function per thing that happened.
//!
//! Kept out of the service so its methods read as rules, and so every event
//! has the same field names wherever it is raised: `item_id`, `item`,
//! `quantity`, `filter_terms`. Item names are household shopping, not
//! secrets, so they are logged; notes are free text and are not.

use crate::models::db::{GroceryItem, GroceryItemStatus};

/// How an addition ended up on the list.
#[derive(Debug, Clone, Copy)]
pub enum Added {
    /// A new entry was inserted.
    NewEntry,
    /// The quantity was folded into an existing active item.
    Merged,
}

/// An item was added by hand, as a new entry or merged into an existing one.
pub fn added(item: &GroceryItem, how: Added, user_id: &str) {
    tracing::info!(
        item_id = %item.id,
        item = %item.name,
        quantity = item.quantity,
        filter_terms = ?item.filter_terms,
        ?how,
        user_id,
        "grocery item added"
    );
}

/// An addition clashed with an active item and the household is being asked.
pub fn duplicate_asked(name: &str, existing: &GroceryItem) {
    tracing::info!(
        item = name,
        existing_item_id = %existing.id,
        "grocery item already on the list; asking whether to merge"
    );
}

/// An item was edited.
pub fn updated(item: &GroceryItem) {
    tracing::info!(
        item_id = %item.id,
        item = %item.name,
        quantity = item.quantity,
        filter_terms = ?item.filter_terms,
        has_note = item.note.is_some(),
        "grocery item updated"
    );
}

/// An item was taken off the list.
pub fn removed(item: &GroceryItem) {
    tracing::info!(item_id = %item.id, item = %item.name, "grocery item removed");
}

/// A change was refused because the item is committed for purchase.
pub fn locked(item: &GroceryItem) {
    tracing::warn!(
        item_id = %item.id,
        item = %item.name,
        "refused to change a committed item; the list must be released first"
    );
}

/// The whole list moved between statuses: a commit or a release.
pub fn list_moved(from: GroceryItemStatus, to: GroceryItemStatus, items: &[GroceryItem]) {
    tracing::info!(?from, ?to, count = items.len(), "grocery list moved");
}
