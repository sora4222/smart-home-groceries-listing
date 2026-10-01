//! What one item's search produced, store by store.

use super::priced::PricedProduct;
use crate::models::db::GroceryItem;
use crate::services::stores::{Store, StoreError};

/// The search for one list item across every store.
#[derive(Debug)]
pub struct ItemSearch {
    pub item: GroceryItem,
    /// The text each store was asked for.
    pub query: String,
    /// One entry per store, in display order.
    pub stores: Vec<StoreOutcome>,
}

/// One store's part of an [`ItemSearch`].
#[derive(Debug)]
pub struct StoreOutcome {
    pub store: Store,
    /// Products after the item's chips were applied, cheapest first.
    /// Empty when the store failed.
    pub products: Vec<PricedProduct>,
    /// Why the store produced nothing, if it failed.
    pub error: Option<StoreError>,
}
