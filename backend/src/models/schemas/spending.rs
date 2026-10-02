//! Bodies for the spending analysis. Money is a decimal string.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::PurchaseOrderResponse;
use crate::models::db::Purchase;
use crate::services::spending::breakdown::{CategorySpend, ItemSpend, PeriodSpend, StoreSpend};
use crate::services::spending::{Period, SpendingReport};
use crate::services::stores::Store;

/// Longest item search or category accepted, matching the columns' CHECKs.
pub const MAX_ITEM_SEARCH_LEN: u64 = 200;
pub const MAX_CATEGORY_LEN: u64 = 100;
/// Longest IANA time zone name accepted ("America/Argentina/ComodRivadavia").
pub const MAX_TIME_ZONE_LEN: u64 = 64;

/// `GET /api/spending` query. Every filter is optional.
#[derive(Debug, Deserialize, Validate)]
pub struct SpendingParams {
    /// First day, in the household's calendar.
    pub from: Option<NaiveDate>,
    /// Last day, included.
    pub to: Option<NaiveDate>,
    pub store: Option<Store>,
    #[validate(length(max = MAX_ITEM_SEARCH_LEN))]
    pub item: Option<String>,
    #[validate(length(max = MAX_CATEGORY_LEN))]
    pub category: Option<String>,
    /// How "Over time" groups spend. Month when absent.
    #[serde(default)]
    pub period: Period,
    /// The browser's time zone, e.g. `Australia/Sydney`. UTC when absent.
    #[validate(length(max = MAX_TIME_ZONE_LEN))]
    pub tz: Option<String>,
}

/// `GET /api/spending/item-prices` query.
#[derive(Debug, Deserialize, Validate)]
pub struct ItemPricesParams {
    #[validate(length(min = 1, max = MAX_ITEM_SEARCH_LEN))]
    pub name: String,
}

/// Every view of the Analysis page.
#[derive(Debug, Serialize)]
pub struct SpendingResponse {
    /// The newest shop, whatever the filters say.
    pub latest_order: Option<PurchaseOrderResponse>,
    /// Every category any purchase is in, A to Z.
    pub categories: Vec<String>,
    pub items_total: Decimal,
    pub delivery_total: Decimal,
    /// Items plus delivery.
    pub total: Decimal,
    pub over_time: Vec<PeriodSpendResponse>,
    pub by_item: Vec<ItemSpendResponse>,
    pub by_store: Vec<StoreSpendResponse>,
    pub by_category: Vec<CategorySpendResponse>,
}

/// One week, month or quarter.
#[derive(Debug, Serialize)]
pub struct PeriodSpendResponse {
    /// Its first day.
    pub start: NaiveDate,
    pub total: Decimal,
}

/// One list item.
#[derive(Debug, Serialize)]
pub struct ItemSpendResponse {
    pub item_name: String,
    pub quantity: i64,
    pub times_bought: usize,
    pub total: Decimal,
}

/// One store.
#[derive(Debug, Serialize)]
pub struct StoreSpendResponse {
    pub store: Store,
    pub store_name: &'static str,
    pub items_total: Decimal,
    pub delivery_total: Decimal,
    pub total: Decimal,
}

/// One category.
#[derive(Debug, Serialize)]
pub struct CategorySpendResponse {
    pub category: String,
    pub total: Decimal,
}

/// What an item cost on one shop.
#[derive(Debug, Serialize)]
pub struct ItemPriceResponse {
    pub bought_at: DateTime<Utc>,
    pub store: Store,
    pub store_name: &'static str,
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub total_price: Decimal,
}

impl From<SpendingReport> for SpendingResponse {
    fn from(report: SpendingReport) -> Self {
        let b = report.breakdown;
        Self {
            latest_order: report.latest_order.map(Into::into),
            categories: report.categories,
            total: b.items_total + b.delivery_total,
            items_total: b.items_total,
            delivery_total: b.delivery_total,
            over_time: b.over_time.into_iter().map(Into::into).collect(),
            by_item: b.by_item.into_iter().map(Into::into).collect(),
            by_store: b.by_store.into_iter().map(Into::into).collect(),
            by_category: b.by_category.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PeriodSpend> for PeriodSpendResponse {
    fn from(spend: PeriodSpend) -> Self {
        Self {
            start: spend.start,
            total: spend.total,
        }
    }
}

impl From<ItemSpend> for ItemSpendResponse {
    fn from(spend: ItemSpend) -> Self {
        Self {
            item_name: spend.item_name,
            quantity: spend.quantity,
            times_bought: spend.times_bought,
            total: spend.total,
        }
    }
}

impl From<StoreSpend> for StoreSpendResponse {
    fn from(spend: StoreSpend) -> Self {
        Self {
            total: spend.total(),
            store: spend.store,
            store_name: spend.store.display_name(),
            items_total: spend.items_total,
            delivery_total: spend.delivery_total,
        }
    }
}

impl From<CategorySpend> for CategorySpendResponse {
    fn from(spend: CategorySpend) -> Self {
        Self {
            category: spend.category,
            total: spend.total,
        }
    }
}

impl From<Purchase> for ItemPriceResponse {
    fn from(purchase: Purchase) -> Self {
        Self {
            bought_at: purchase.bought_at,
            store: purchase.store,
            store_name: purchase.store.display_name(),
            product_name: purchase.product_name,
            brand: purchase.brand,
            package_size: purchase.package_size,
            quantity: purchase.quantity,
            unit_price: purchase.unit_price,
            total_price: purchase.total_price,
        }
    }
}
