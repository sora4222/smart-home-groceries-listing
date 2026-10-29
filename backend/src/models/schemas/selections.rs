//! Bodies for choosing products for list items.
//!
//! The request names only the store and the store's product id; every other
//! detail in the response came from the store's search answer. Money is a
//! decimal string, as everywhere else.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::db::ItemSelection;
use crate::services::selections::{Counted, ItemChoices};
use crate::services::stores::{Basis, Store};

/// Longest store product id accepted, matching the `product_id` CHECK.
pub const MAX_PRODUCT_ID_LEN: u64 = 100;

/// `PUT /api/grocery-items/{id}/selection`: the product picked in the
/// item's price comparison. An unknown store fails to parse (422).
#[derive(Debug, Deserialize, Validate)]
pub struct ItemSelectionChoose {
    pub store: Store,
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: String,
}

/// Most items one `PUT /api/order-stores` may switch.
pub const MAX_ORDER_STORE_PICKS: u64 = 500;

/// `PUT /api/order-stores`: which store each item's order buys from. Every
/// item must already have a product chosen at its store.
#[derive(Debug, Deserialize, Validate)]
pub struct OrderStoresSet {
    #[validate(length(min = 1, max = MAX_ORDER_STORE_PICKS))]
    pub picks: Vec<OrderStorePick>,
}

/// One item and the store its order buys from.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderStorePick {
    pub grocery_item_id: Uuid,
    pub store: Store,
}

/// `DELETE /api/grocery-items/{id}/selection?store=&product_id=`: what to
/// clear. A product id clears that product at the store; a store alone
/// clears every product there; nothing clears every choice.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct ItemSelectionClear {
    pub store: Option<Store>,
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: Option<String>,
}

/// `PUT /api/grocery-items/{id}/selection/quantity`: how many of one of the
/// item's chosen products to buy.
#[derive(Debug, Deserialize, Validate)]
pub struct ItemSelectionQuantity {
    pub store: Store,
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: String,
    #[validate(range(min = 1, max = 999))]
    pub quantity: i32,
}

/// `POST /api/grocery-items/{id}/selection/snapshot`: the saved snapshot.
#[derive(Debug, Serialize)]
pub struct SelectionSnapshotResponse {
    pub id: Uuid,
}

/// `POST /api/selection-snapshots/{id}/restore?grocery_item_id=`: the item
/// to put the products on, when not the one the snapshot was taken of.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct SelectionRestoreQuery {
    pub grocery_item_id: Option<Uuid>,
}

/// One item's choices in `GET /api/item-selections`: the oldest product the
/// order buys, the order's other products for it, and the item's choices at
/// the other stores.
#[derive(Debug, Serialize)]
pub struct ItemChoicesResponse {
    #[serde(flatten)]
    pub chosen: ItemSelectionResponse,
    /// More products the order buys for the item, at the same store.
    pub bought_with: Vec<ItemSelectionResponse>,
    /// The item's choices at other stores, which the order does not buy now.
    pub also_chosen: Vec<ItemSelectionResponse>,
}

impl From<ItemChoices> for ItemChoicesResponse {
    fn from(choices: ItemChoices) -> Self {
        Self {
            chosen: choices.for_order.into(),
            bought_with: choices.bought_with.into_iter().map(Into::into).collect(),
            also_chosen: choices.others.into_iter().map(Into::into).collect(),
        }
    }
}

/// A saved choice, as the web app and the order screen read it.
#[derive(Debug, Serialize)]
pub struct ItemSelectionResponse {
    pub grocery_item_id: Uuid,
    pub store: Store,
    pub store_name: &'static str,
    pub product_id: String,
    pub name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    /// Shelf price for one when it was chosen.
    pub price: Option<Decimal>,
    pub unit_price: Option<SelectedUnitPrice>,
    /// What `priced_quantity` cost when it was chosen, best deal applied.
    pub total_price: Option<Decimal>,
    pub priced_quantity: i32,
    pub url: String,
    pub image_url: Option<String>,
    pub selected_by: String,
    pub selected_at: DateTime<Utc>,
    /// How many of this product the order buys.
    pub quantity: i32,
}

/// The normalised unit price saved with a choice.
#[derive(Debug, Serialize)]
pub struct SelectedUnitPrice {
    pub amount: Decimal,
    pub per: Basis,
}

impl From<Counted> for ItemSelectionResponse {
    fn from(counted: Counted) -> Self {
        let mut response = Self::from(counted.selection);
        response.quantity = counted.quantity;
        response
    }
}

impl From<ItemSelection> for ItemSelectionResponse {
    /// A choice read on its own: its stored count, or the quantity it was
    /// priced for.
    fn from(row: ItemSelection) -> Self {
        let unit_price = row
            .unit_price
            .zip(row.unit_price_per)
            .map(|(amount, per)| SelectedUnitPrice { amount, per });
        Self {
            grocery_item_id: row.grocery_item_id,
            store: row.store,
            store_name: row.store.display_name(),
            product_id: row.product_id,
            name: row.product_name,
            brand: row.brand,
            package_size: row.package_size,
            price: row.price,
            unit_price,
            total_price: row.total_price,
            priced_quantity: row.priced_quantity,
            url: row.url,
            image_url: row.image_url,
            selected_by: row.selected_by,
            selected_at: row.selected_at,
            quantity: row.quantity.unwrap_or(row.priced_quantity),
        }
    }
}
