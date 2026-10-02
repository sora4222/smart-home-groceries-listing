//! Row types for `purchase_orders` and `purchases`, and the enumerations
//! their TEXT + CHECK columns hold (migration `20261002120000`).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::db::GroceryItemStatus;
use crate::services::stores::Store;

/// How the app learned that a shop was bought (`purchase_orders.source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PurchaseSource {
    /// The "Fill trolley" bookmarklet reported that it filled the trolley.
    TrolleyFill,
}

/// Where a purchase's category came from (`purchases.category_source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CategorySource {
    /// Mapped from the store's own category for the product.
    Store,
    /// Guessed from the product's name.
    Name,
    /// No keyword matched; shown as "Other".
    None,
}

/// One shop at one store (`purchase_orders`).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PurchaseOrder {
    pub id: Uuid,
    pub store: Store,
    /// The sum of the purchases' `total_price`.
    pub items_total: Decimal,
    pub delivery_fee: Decimal,
    /// The household member who sent the products to the store.
    pub recorded_by: String,
    pub source: PurchaseSource,
    pub trolley_handoff_id: Option<Uuid>,
    pub bought_at: DateTime<Utc>,
}

impl PurchaseOrder {
    /// Items plus delivery.
    pub fn total(&self) -> Decimal {
        self.items_total + self.delivery_fee
    }
}

/// One product bought for one list item (`purchases`). A snapshot: it
/// outlives the list item.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Purchase {
    pub id: Uuid,
    pub order_id: Uuid,
    /// `None` once the list item is deleted.
    pub grocery_item_id: Option<Uuid>,
    pub item_name: String,
    /// `item_name` lowercased with spaces collapsed: the analysis groups by it.
    pub item_key: String,
    /// The list item's status before it became `ordered`, for Undo.
    pub item_status_before: GroceryItemStatus,
    pub store: Store,
    pub product_id: String,
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub quantity: i32,
    /// What one cost, deals applied.
    pub unit_price: Decimal,
    /// The store's price for one, when it showed one.
    pub shelf_price: Option<Decimal>,
    /// What `quantity` cost, deals applied.
    pub total_price: Decimal,
    /// This line's part of the order's delivery fee.
    pub delivery_fee_share: Decimal,
    pub store_category: Option<String>,
    pub category: String,
    pub category_source: CategorySource,
    pub bought_at: DateTime<Utc>,
}

impl Purchase {
    /// What this line cost the household, delivery share included.
    pub fn spend(&self) -> Decimal {
        self.total_price + self.delivery_fee_share
    }
}
