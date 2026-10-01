//! Bodies for a list item's product search.
//!
//! Money is sent as decimal strings (`"4.95"`), never JSON numbers, so the
//! web app shows exactly what the store charged.

use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::services::product_search::{ItemSearch, PricedProduct, StoreOutcome};
use crate::services::stores::{Deal, Store, UnitPrice};

/// `GET /api/grocery-items/{id}/products`.
#[derive(Debug, Serialize)]
pub struct ItemProductsResponse {
    pub item: SearchedItem,
    /// What each store was asked for: the name and chips together.
    pub query: String,
    pub stores: Vec<StoreProductsResponse>,
}

/// The item the products were found for.
#[derive(Debug, Serialize)]
pub struct SearchedItem {
    pub id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub filter_terms: Vec<String>,
}

/// One store's results.
#[derive(Debug, Serialize)]
pub struct StoreProductsResponse {
    pub store: Store,
    pub store_name: &'static str,
    /// `ok`, or why the store produced nothing: `blocked`, `unreachable`,
    /// `unexpected_response`.
    pub status: &'static str,
    /// A sentence to show when the store failed.
    pub message: Option<String>,
    pub products: Vec<ProductResponse>,
}

/// One product, priced for the item's quantity.
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product_id: String,
    pub name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub price: Option<Decimal>,
    pub was_price: Option<Decimal>,
    pub on_special: bool,
    pub unit_price: Option<UnitPrice>,
    /// Why the unit price needs care: converted, on another basis, missing.
    pub unit_price_note: Option<String>,
    pub deals: Vec<Deal>,
    /// The cost of the item's quantity with the best deal applied.
    pub total_price: Option<Decimal>,
    pub deal_applied: bool,
    pub category: Option<String>,
    pub url: String,
    pub available: bool,
}

impl From<ItemSearch> for ItemProductsResponse {
    fn from(search: ItemSearch) -> Self {
        Self {
            item: SearchedItem {
                id: search.item.id,
                name: search.item.name,
                quantity: search.item.quantity,
                filter_terms: search.item.filter_terms,
            },
            query: search.query,
            stores: search.stores.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<StoreOutcome> for StoreProductsResponse {
    fn from(outcome: StoreOutcome) -> Self {
        let name = outcome.store.display_name();
        Self {
            store: outcome.store,
            store_name: name,
            status: outcome.error.as_ref().map_or("ok", |err| err.kind()),
            message: outcome.error.as_ref().map(|err| err.user_message(name)),
            products: outcome.products.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PricedProduct> for ProductResponse {
    fn from(priced: PricedProduct) -> Self {
        let p = priced.product;
        Self {
            product_id: p.product_id,
            name: p.name,
            brand: p.brand,
            package_size: p.package_size,
            price: p.price,
            was_price: p.was_price,
            on_special: p.on_special,
            unit_price: p.unit_price,
            unit_price_note: priced.unit_price_note,
            deals: p.deals,
            total_price: priced.total_price,
            deal_applied: priced.deal_applied,
            category: p.category,
            url: p.url,
            available: p.available,
        }
    }
}
