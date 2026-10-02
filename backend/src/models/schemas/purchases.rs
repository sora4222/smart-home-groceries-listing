//! Bodies for the purchase history: saved shops, Undo, and a product's past
//! purchases. Money is a decimal string, as everywhere else.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::MAX_PRODUCT_ID_LEN;
use crate::models::db::{Purchase, PurchaseOrder, PurchaseSource};
use crate::services::purchases::{OrderSummary, ProductHistory};
use crate::services::stores::Store;

/// Most products one lookup may ask about (a comparison shows far fewer).
pub const MAX_LOOKUP_PRODUCTS: u64 = 200;

/// A saved shop.
#[derive(Debug, Serialize)]
pub struct PurchaseOrderResponse {
    pub id: Uuid,
    pub store: Store,
    pub store_name: &'static str,
    pub source: PurchaseSource,
    pub trolley_handoff_id: Option<Uuid>,
    pub bought_at: DateTime<Utc>,
    pub items_total: Decimal,
    pub delivery_fee: Decimal,
    /// Items plus delivery.
    pub total: Decimal,
    /// How many products it holds; `None` where it was not counted.
    pub products: Option<i64>,
}

impl From<PurchaseOrder> for PurchaseOrderResponse {
    fn from(order: PurchaseOrder) -> Self {
        Self {
            total: order.total(),
            id: order.id,
            store: order.store,
            store_name: order.store.display_name(),
            source: order.source,
            trolley_handoff_id: order.trolley_handoff_id,
            bought_at: order.bought_at,
            items_total: order.items_total,
            delivery_fee: order.delivery_fee,
            products: None,
        }
    }
}

impl From<OrderSummary> for PurchaseOrderResponse {
    fn from(summary: OrderSummary) -> Self {
        Self {
            products: Some(summary.products),
            ..summary.order.into()
        }
    }
}

/// `DELETE /api/purchase-orders/{id}`: what Undo changed.
#[derive(Debug, Serialize)]
pub struct UndoPurchaseResponse {
    /// List items put back on the list.
    pub items_restored: usize,
}

/// `POST /api/purchase-history/recategorise`: how many purchases changed.
#[derive(Debug, Serialize)]
pub struct RecategoriseResponse {
    pub changed: usize,
}

/// One product a lookup asks about.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ProductRef {
    pub store: Store,
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: String,
}

/// `POST /api/purchase-history/products`: the products a comparison shows.
#[derive(Debug, Deserialize, Validate)]
pub struct ProductHistoryLookup {
    #[validate(length(max = MAX_LOOKUP_PRODUCTS), nested)]
    pub products: Vec<ProductRef>,
}

/// One product's past purchases. Products never bought are left out.
#[derive(Debug, Serialize)]
pub struct ProductHistoryResponse {
    pub store: Store,
    pub product_id: String,
    pub times_bought: usize,
    /// Newest first.
    pub purchases: Vec<PastPurchaseResponse>,
}

/// One past purchase of a product.
#[derive(Debug, Serialize)]
pub struct PastPurchaseResponse {
    pub bought_at: DateTime<Utc>,
    pub quantity: i32,
    /// What one cost, deals applied.
    pub unit_price: Decimal,
    pub total_price: Decimal,
}

impl From<ProductHistory> for ProductHistoryResponse {
    fn from(history: ProductHistory) -> Self {
        Self {
            times_bought: history.times_bought(),
            store: history.store,
            product_id: history.product_id,
            purchases: history.purchases.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Purchase> for PastPurchaseResponse {
    fn from(purchase: Purchase) -> Self {
        Self {
            bought_at: purchase.bought_at,
            quantity: purchase.quantity,
            unit_price: purchase.unit_price,
            total_price: purchase.total_price,
        }
    }
}
