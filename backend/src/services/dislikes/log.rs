//! Dislikes' log events, one function per thing that happened.
//!
//! Member ids, stores, product ids and names are logged. Member emails are
//! not; the saved display name is enough to follow a dislike.

use uuid::Uuid;

use crate::models::db::{DislikeOverride, ProductDislike};
use crate::services::stores::Store;

/// A member disliked a product, or refreshed an earlier dislike.
pub fn disliked(dislike: &ProductDislike) {
    tracing::info!(
        user_id = %dislike.user_id,
        store = %dislike.store,
        product_id = %dislike.product_id,
        product = %dislike.product_name,
        "product disliked"
    );
}

/// A member removed their own dislike (the permanent override).
pub fn removed(user_id: &str, store: Store, product_id: &str, had_dislike: bool) {
    if had_dislike {
        tracing::info!(user_id, store = %store, product_id, "product dislike removed");
    } else {
        tracing::debug!(user_id, store = %store, product_id, "no product dislike to remove");
    }
}

/// A member chose to buy a disliked product for this order anyway.
pub fn overridden(item: &DislikeOverride) {
    tracing::info!(
        item_id = %item.grocery_item_id,
        store = %item.store,
        product_id = %item.product_id,
        user_id = %item.overridden_by,
        "dislike overridden for this order"
    );
}

/// An override was taken back, so the dislikes count again for the item.
pub fn override_cleared(item_id: Uuid, store: Store, product_id: &str, had_override: bool) {
    if had_override {
        tracing::info!(item_id = %item_id, store = %store, product_id, "dislike override cleared");
    } else {
        tracing::debug!(item_id = %item_id, store = %store, product_id, "no dislike override to clear");
    }
}

/// An override was refused: nobody dislikes the product, or the item is gone.
pub fn override_refused(item_id: Uuid, store: Store, product_id: &str, reason: &str) {
    tracing::warn!(item_id = %item_id, store = %store, product_id, reason, "dislike override refused");
}
