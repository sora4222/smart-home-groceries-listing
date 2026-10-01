//! Product search's log events, one function per thing that happened.
//!
//! Item names and queries are logged; nothing from a store's response
//! beyond counts, and never cookies or upstream bodies.

use uuid::Uuid;

use crate::services::stores::{Store, StoreError};

/// A store answered an item's search.
pub fn store_answered(item_id: Uuid, store: Store, query: &str, found: usize, kept: usize) {
    tracing::info!(
        item_id = %item_id,
        %store,
        query,
        found,
        kept_after_filters = kept,
        "store search answered"
    );
}

/// A store failed an item's search. The other stores' results still show.
pub fn store_failed(item_id: Uuid, store: Store, query: &str, err: &StoreError) {
    tracing::warn!(
        item_id = %item_id,
        %store,
        query,
        kind = err.kind(),
        %err,
        "store search failed"
    );
}
