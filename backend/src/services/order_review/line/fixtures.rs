//! A committed item and its saved choice, for this module's tests.

use super::*;
use crate::models::db::{GroceryItemSource, GroceryItemStatus};
use crate::services::stores::Store;

/// A committed list item.
pub fn item(name: &str, quantity: i32) -> GroceryItem {
    GroceryItem {
        id: uuid::Uuid::new_v4(),
        name: name.into(),
        quantity,
        status: GroceryItemStatus::Committed,
        source: GroceryItemSource::Manual,
        note: None,
        filter_terms: Vec::new(),
        added_by_user_id: None,
        created_at: chrono::Utc::now(),
    }
}

/// A choice of `product_id` at `store`, made when one cost `price`.
pub fn choice(item: &GroceryItem, store: Store, product_id: &str, price: Decimal) -> ItemSelection {
    ItemSelection {
        id: uuid::Uuid::new_v4(),
        grocery_item_id: item.id,
        store,
        product_id: product_id.into(),
        product_name: product_id.into(),
        brand: None,
        package_size: None,
        price: Some(price),
        unit_price: None,
        unit_price_per: None,
        total_price: Some(price),
        priced_quantity: 1,
        url: String::new(),
        selected_by: "user".into(),
        selected_at: chrono::Utc::now(),
    }
}
