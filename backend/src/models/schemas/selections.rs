//! Bodies for choosing one product per list item.
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
    pub selected_by: String,
    pub selected_at: DateTime<Utc>,
}

/// The normalised unit price saved with a choice.
#[derive(Debug, Serialize)]
pub struct SelectedUnitPrice {
    pub amount: Decimal,
    pub per: Basis,
}

impl From<ItemSelection> for ItemSelectionResponse {
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
            selected_by: row.selected_by,
            selected_at: row.selected_at,
        }
    }
}
