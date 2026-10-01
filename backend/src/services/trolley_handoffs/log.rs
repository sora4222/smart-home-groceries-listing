//! Trolley handoff log events, one function per thing that happened.
//!
//! Handoff ids, stores, counts and product ids are logged. The store-tab
//! secret never is.

use uuid::Uuid;

use crate::models::db::{TrolleyHandoff, TrolleyHandoffLine};
use crate::services::stores::Store;

/// The web app created a handoff.
pub fn created(handoff: &TrolleyHandoff, lines: usize, replaced: u64) {
    tracing::info!(
        handoff_id = %handoff.id,
        store = %handoff.store,
        lines,
        replaced_waiting_handoffs = replaced,
        user_id = %handoff.created_by,
        expires_at = %handoff.expires_at,
        "trolley handoff created"
    );
}

/// Nothing on the list has a product chosen at this store.
pub fn nothing_to_hand_off(store: Store) {
    tracing::info!(store = %store, "trolley handoff refused: no chosen products at this store");
}

/// The store tab claimed a handoff.
pub fn claimed(handoff: &TrolleyHandoff, products: usize) {
    tracing::info!(handoff_id = %handoff.id, store = %handoff.store, products, "trolley handoff claimed by the store tab");
}

/// The store tab asked for work but none was waiting (or it had expired).
pub fn nothing_to_claim(store: Store) {
    tracing::info!(store = %store, "store tab found no waiting trolley handoff");
}

/// The store tab reported back.
pub fn reported(handoff: &TrolleyHandoff, lines: &[TrolleyHandoffLine]) {
    for line in lines.iter().filter(|l| l.problem.is_some()) {
        tracing::warn!(
            handoff_id = %handoff.id,
            product_id = %line.product_id,
            product = %line.product_name,
            problem = line.problem.as_deref().unwrap_or_default(),
            "store refused a trolley line"
        );
    }
    tracing::info!(handoff_id = %handoff.id, status = ?handoff.status, "trolley handoff report recorded");
}

/// A report arrived for a handoff that was not waiting for one.
pub fn report_refused(handoff_id: Uuid, reason: &str) {
    tracing::warn!(handoff_id = %handoff_id, reason, "trolley handoff report refused");
}
