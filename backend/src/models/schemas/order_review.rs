//! The body of `GET /api/order-review`.
//!
//! Money is a decimal string, as everywhere else. `chosen_*` fields are what
//! the store showed when the product was chosen; the others are today's.

use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::models::db::GroceryItem;
use crate::services::order_review::{LineStatus, OrderLine, OrderReview, PriceChange, StoreOrder};
use crate::services::stores::Store;

/// The order as it stands right now.
#[derive(Debug, Serialize)]
pub struct OrderReviewResponse {
    /// Stores with something to buy, Woolworths first.
    pub stores: Vec<StoreOrderResponse>,
    /// Committed items with no product chosen.
    pub unchosen: Vec<UnchosenItemResponse>,
    /// Item costs only — delivery fees are not included.
    pub total: Decimal,
    /// Every item has a product with a price today.
    pub complete: bool,
}

/// Everything bought at one store.
#[derive(Debug, Serialize)]
pub struct StoreOrderResponse {
    pub store: Store,
    pub store_name: &'static str,
    pub lines: Vec<OrderLineResponse>,
    pub subtotal: Decimal,
    pub complete: bool,
}

/// One item of the order.
#[derive(Debug, Serialize)]
pub struct OrderLineResponse {
    pub grocery_item_id: Uuid,
    pub item_name: String,
    /// The item's quantity now — what will be bought.
    pub quantity: i32,
    pub product_id: String,
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub url: String,
    pub status: LineStatusBody,
    /// Shelf price for one today.
    pub price: Option<Decimal>,
    /// `quantity` today, best deal applied.
    pub total_price: Option<Decimal>,
    pub deal_applied: bool,
    pub price_change: Option<PriceChangeBody>,
    pub chosen_price: Option<Decimal>,
    pub chosen_total_price: Option<Decimal>,
    pub chosen_priced_quantity: i32,
    pub problem: Option<String>,
}

/// [`LineStatus`] on the wire.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineStatusBody {
    Priced,
    Unavailable,
    NotOffered,
    StoreFailed,
}

/// [`PriceChange`] on the wire.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceChangeBody {
    Same,
    Up,
    Down,
}

/// A committed item still waiting for its product.
#[derive(Debug, Serialize)]
pub struct UnchosenItemResponse {
    pub grocery_item_id: Uuid,
    pub name: String,
    pub quantity: i32,
}

impl From<OrderReview> for OrderReviewResponse {
    fn from(review: OrderReview) -> Self {
        Self {
            stores: review.stores.into_iter().map(Into::into).collect(),
            unchosen: review.unchosen.into_iter().map(Into::into).collect(),
            total: review.total,
            complete: review.complete,
        }
    }
}

impl From<StoreOrder> for StoreOrderResponse {
    fn from(order: StoreOrder) -> Self {
        Self {
            store: order.store,
            store_name: order.store.display_name(),
            lines: order.lines.into_iter().map(Into::into).collect(),
            subtotal: order.subtotal,
            complete: order.complete,
        }
    }
}

impl From<OrderLine> for OrderLineResponse {
    fn from(line: OrderLine) -> Self {
        let choice = line.choice;
        Self {
            grocery_item_id: line.item.id,
            item_name: line.item.name,
            quantity: line.item.quantity,
            product_id: choice.product_id,
            product_name: choice.product_name,
            brand: choice.brand,
            package_size: choice.package_size,
            url: choice.url,
            status: line.status.into(),
            price: line.price,
            total_price: line.total_price,
            deal_applied: line.deal_applied,
            price_change: line.price_change.map(Into::into),
            chosen_price: choice.price,
            chosen_total_price: choice.total_price,
            chosen_priced_quantity: choice.priced_quantity,
            problem: line.problem,
        }
    }
}

impl From<LineStatus> for LineStatusBody {
    fn from(status: LineStatus) -> Self {
        match status {
            LineStatus::Priced => Self::Priced,
            LineStatus::Unavailable => Self::Unavailable,
            LineStatus::NotOffered => Self::NotOffered,
            LineStatus::StoreFailed => Self::StoreFailed,
        }
    }
}

impl From<PriceChange> for PriceChangeBody {
    fn from(change: PriceChange) -> Self {
        match change {
            PriceChange::Same => Self::Same,
            PriceChange::Up => Self::Up,
            PriceChange::Down => Self::Down,
        }
    }
}

impl From<GroceryItem> for UnchosenItemResponse {
    fn from(item: GroceryItem) -> Self {
        Self {
            grocery_item_id: item.id,
            name: item.name,
            quantity: item.quantity,
        }
    }
}
