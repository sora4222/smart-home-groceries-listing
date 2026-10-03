//! Purchase history log events, one function per thing that happened.
//!
//! Order and handoff ids, stores, counts and totals are logged. Nothing here
//! is a secret.

use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{PurchaseOrder, TrolleyHandoff, TrolleyHandoffLine};

/// A filled trolley was saved as bought.
pub fn saved(order: &PurchaseOrder, purchases: usize, items_moved: usize) {
    tracing::info!(
        order_id = %order.id,
        handoff_id = ?order.trolley_handoff_id,
        store = %order.store,
        purchases,
        items_moved_to_ordered = items_moved,
        items_total = %order.items_total,
        delivery_fee = %order.delivery_fee,
        "purchase saved from trolley fill"
    );
}

/// A reported handoff had nothing that could be saved.
pub fn nothing_to_save(handoff: &TrolleyHandoff, added_lines: usize) {
    tracing::info!(
        handoff_id = %handoff.id,
        store = %handoff.store,
        added_lines,
        "trolley fill not saved as a purchase: nothing to save"
    );
}

/// One added product was not saved.
pub fn line_skipped(line: &TrolleyHandoffLine, why: &str) {
    tracing::warn!(
        handoff_id = %line.handoff_id,
        item_id = %line.grocery_item_id,
        product_id = %line.product_id,
        why,
        "trolley line not saved as a purchase"
    );
}

/// The same handoff was saved before; nothing new was written.
pub fn already_saved(handoff_id: Uuid) {
    tracing::debug!(%handoff_id, "trolley fill already saved as a purchase");
}

/// Saving a trolley fill failed after the report was accepted.
pub fn save_failed(handoff_id: Uuid, err: &ApiError) {
    tracing::error!(%handoff_id, error = %err, "saving a trolley fill as a purchase failed");
}

/// A household member undid a saved shop.
pub fn undone(order: &PurchaseOrder, items_restored: usize, user_id: &str) {
    tracing::info!(
        order_id = %order.id,
        store = %order.store,
        items_restored,
        user_id,
        "purchase undone"
    );
}

/// Categories were worked out again for every purchase.
pub fn recategorised(purchases: usize, changed: usize) {
    tracing::info!(purchases, changed, "purchases re-categorised");
}
