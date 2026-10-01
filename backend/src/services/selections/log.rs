//! Product choice's log events, one function per thing that happened.
//!
//! Item ids, stores, product ids and names are logged; prices are logged as
//! the store showed them. Nothing here is a secret.

use uuid::Uuid;

use crate::models::db::ItemSelection;

/// A product was chosen for an item, or a new one replaced the old.
pub fn chosen(selection: &ItemSelection) {
    tracing::info!(
        item_id = %selection.grocery_item_id,
        store = %selection.store,
        product_id = %selection.product_id,
        product = %selection.product_name,
        price = ?selection.price,
        total_price = ?selection.total_price,
        priced_quantity = selection.priced_quantity,
        user_id = %selection.selected_by,
        "product chosen for item"
    );
}

/// A choice was refused: the store failed, or no longer offers the product.
pub fn refused(item_id: Uuid, store: &str, product_id: &str, reason: &str) {
    tracing::warn!(item_id = %item_id, store, product_id, reason, "product choice refused");
}

/// A household member cleared an item's choice.
pub fn cleared(item_id: Uuid, had_choice: bool) {
    if had_choice {
        tracing::info!(item_id = %item_id, "product choice cleared");
    } else {
        tracing::debug!(item_id = %item_id, "no product choice to clear");
    }
}

/// An edit renamed an item or changed its chips, so its choice was dropped.
pub fn forgotten_after_edit(item_id: Uuid, item: &str) {
    tracing::info!(
        item_id = %item_id,
        item,
        "product choice dropped: the item was renamed or its chips changed"
    );
}

/// The item changed between the store search and saving the choice.
pub fn item_changed_while_choosing(item_id: Uuid) {
    tracing::warn!(item_id = %item_id, "item changed while a product was being chosen");
}
