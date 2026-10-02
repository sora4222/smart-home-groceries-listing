//! The four views of the spending analysis, worked out from purchases.
//!
//! * Over time — spend per week, month or quarter, gaps filled with zero.
//! * By item — spend per list item (grouped by its name, any case or spacing).
//! * By store — items and delivery for each store.
//! * By category — spend per category.
//!
//! "Spend" is what the products cost. Delivery fees are their own line in the
//! totals and in "By store", and are included in "Over time", because a fee
//! belongs to a shop, not to an item or a category. Pure: no I/O.

use std::collections::HashMap;

use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_decimal::Decimal;

use super::period::Period;
use super::range::local_date;
use crate::models::db::Purchase;
use crate::services::stores::Store;

/// The most buckets "Over time" fills in, so a typo'd range stays small.
const MAX_BUCKETS: usize = 600;

/// Spend in one week, month or quarter, delivery included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeriodSpend {
    pub start: NaiveDate,
    pub total: Decimal,
}

/// Spend on one list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSpend {
    pub item_key: String,
    /// The name it had when last bought.
    pub item_name: String,
    pub quantity: i64,
    pub times_bought: usize,
    pub total: Decimal,
}

/// Spend at one store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSpend {
    pub store: Store,
    pub items_total: Decimal,
    pub delivery_total: Decimal,
}

impl StoreSpend {
    /// Items plus delivery.
    pub fn total(&self) -> Decimal {
        self.items_total + self.delivery_total
    }
}

/// Spend in one category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorySpend {
    pub category: String,
    pub total: Decimal,
}

/// Every view over the same purchases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breakdown {
    pub items_total: Decimal,
    pub delivery_total: Decimal,
    pub over_time: Vec<PeriodSpend>,
    pub by_item: Vec<ItemSpend>,
    pub by_store: Vec<StoreSpend>,
    pub by_category: Vec<CategorySpend>,
}

/// Builds every view from `purchases` (oldest first). `stores` are the
/// stores "By store" lists, even with nothing spent there.
pub fn build(purchases: &[Purchase], period: Period, tz: Tz, stores: &[Store]) -> Breakdown {
    Breakdown {
        items_total: purchases.iter().map(|p| p.total_price).sum(),
        delivery_total: purchases.iter().map(|p| p.delivery_fee_share).sum(),
        over_time: over_time(purchases, period, tz),
        by_item: by_item(purchases),
        by_store: by_store(purchases, stores),
        by_category: by_category(purchases),
    }
}

/// Spend per bucket, from the first bucket with a purchase to the last.
fn over_time(purchases: &[Purchase], period: Period, tz: Tz) -> Vec<PeriodSpend> {
    let mut sums: HashMap<NaiveDate, Decimal> = HashMap::new();
    for purchase in purchases {
        let start = period.start_of(local_date(tz, purchase.bought_at));
        *sums.entry(start).or_default() += purchase.spend();
    }
    let (Some(first), Some(last)) = (sums.keys().min().copied(), sums.keys().max().copied()) else {
        return Vec::new();
    };
    let mut buckets = Vec::new();
    let mut start = first;
    while start <= last && buckets.len() < MAX_BUCKETS {
        buckets.push(PeriodSpend {
            start,
            total: sums.get(&start).copied().unwrap_or_default(),
        });
        start = period.next(start);
    }
    buckets
}

/// Spend per item, biggest first; ties A to Z.
fn by_item(purchases: &[Purchase]) -> Vec<ItemSpend> {
    let mut items: Vec<ItemSpend> = Vec::new();
    for purchase in purchases {
        match items.iter_mut().find(|i| i.item_key == purchase.item_key) {
            Some(item) => {
                item.item_name.clone_from(&purchase.item_name);
                item.quantity += i64::from(purchase.quantity);
                item.times_bought += 1;
                item.total += purchase.total_price;
            }
            None => items.push(ItemSpend {
                item_key: purchase.item_key.clone(),
                item_name: purchase.item_name.clone(),
                quantity: i64::from(purchase.quantity),
                times_bought: 1,
                total: purchase.total_price,
            }),
        }
    }
    items.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| a.item_key.cmp(&b.item_key))
    });
    items
}

/// Items and delivery at each of `stores`, in that order.
fn by_store(purchases: &[Purchase], stores: &[Store]) -> Vec<StoreSpend> {
    stores
        .iter()
        .map(|&store| {
            let at_store = purchases.iter().filter(|p| p.store == store);
            StoreSpend {
                store,
                items_total: at_store.clone().map(|p| p.total_price).sum(),
                delivery_total: at_store.map(|p| p.delivery_fee_share).sum(),
            }
        })
        .collect()
}

/// Spend per category, biggest first; ties A to Z.
fn by_category(purchases: &[Purchase]) -> Vec<CategorySpend> {
    let mut sums: HashMap<&str, Decimal> = HashMap::new();
    for purchase in purchases {
        *sums.entry(purchase.category.as_str()).or_default() += purchase.total_price;
    }
    let mut categories: Vec<CategorySpend> = sums
        .into_iter()
        .map(|(category, total)| CategorySpend {
            category: category.to_string(),
            total,
        })
        .collect();
    categories.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| a.category.cmp(&b.category))
    });
    categories
}

#[cfg(test)]
mod tests;
