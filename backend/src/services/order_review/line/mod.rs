//! Re-pricing one chosen product for the order.
//!
//! A choice keeps the price the store showed when it was made. Before anything
//! is bought the store is asked again: the product may cost more or less now,
//! the item's quantity may have changed, or the store may no longer sell it.
//! This module turns the store's fresh answer into one [`OrderLine`]. It is
//! pure — no database, no store calls — so every outcome is unit-tested.

use std::cmp::Ordering;

use rust_decimal::Decimal;

use crate::models::db::{GroceryItem, ItemSelection};
use crate::services::product_search::StoreOutcome;

/// Whether the chosen product can be bought as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    /// The store still sells it; the line has today's price.
    Priced,
    /// The store lists it but cannot deliver it now.
    Unavailable,
    /// The store's results for the item no longer include it.
    NotOffered,
    /// The store could not be asked.
    StoreFailed,
}

/// How the shelf price for one compares with the price when it was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceChange {
    Same,
    Up,
    Down,
}

/// One item of the order: what was chosen, and what it costs now.
#[derive(Debug, Clone)]
pub struct OrderLine {
    pub item: GroceryItem,
    pub choice: ItemSelection,
    pub status: LineStatus,
    /// Shelf price for one today; `None` unless priced and shown.
    pub price: Option<Decimal>,
    /// The item's current quantity today, best deal applied.
    pub total_price: Option<Decimal>,
    pub deal_applied: bool,
    /// `None` when either price is unknown.
    pub price_change: Option<PriceChange>,
    /// A sentence for the household when the line cannot be bought as it is.
    pub problem: Option<String>,
}

impl OrderLine {
    /// The amount this line adds to its store's subtotal, if any.
    pub fn counted_total(&self) -> Option<Decimal> {
        match self.status {
            LineStatus::Priced => self.total_price,
            _ => None,
        }
    }
}

/// Re-prices `choice` from the store's fresh answer for `item`.
///
/// `outcome` is `None` when no client is set up for the chosen store.
pub fn reprice(
    item: GroceryItem,
    choice: ItemSelection,
    outcome: Option<StoreOutcome>,
) -> OrderLine {
    let store_name = choice.store.display_name();
    let Some(outcome) = outcome else {
        let problem = format!("{store_name} is not set up, so this price could not be checked.");
        return unpriced(item, choice, LineStatus::StoreFailed, problem);
    };
    if let Some(err) = outcome.error {
        let problem = err.user_message(store_name);
        return unpriced(item, choice, LineStatus::StoreFailed, problem);
    }

    let found = outcome
        .products
        .into_iter()
        .find(|priced| priced.product.product_id == choice.product_id);
    let Some(priced) = found else {
        let problem = format!(
            "{store_name} no longer offers this product for {} — choose another on the list.",
            item.name
        );
        return unpriced(item, choice, LineStatus::NotOffered, problem);
    };
    if !priced.product.available {
        let problem =
            format!("Unavailable at {store_name} right now — choose another on the list.");
        return unpriced(item, choice, LineStatus::Unavailable, problem);
    }

    let price = priced.product.price;
    OrderLine {
        price_change: compare(choice.price, price),
        problem: priced
            .total_price
            .is_none()
            .then(|| format!("{store_name} shows no price for this product right now.")),
        item,
        choice,
        status: LineStatus::Priced,
        price,
        total_price: priced.total_price,
        deal_applied: priced.deal_applied,
    }
}

/// A line that cannot be bought as it is.
fn unpriced(
    item: GroceryItem,
    choice: ItemSelection,
    status: LineStatus,
    problem: String,
) -> OrderLine {
    OrderLine {
        item,
        choice,
        status,
        price: None,
        total_price: None,
        deal_applied: false,
        price_change: None,
        problem: Some(problem),
    }
}

/// Then and now, for one unit at shelf price.
fn compare(then: Option<Decimal>, now: Option<Decimal>) -> Option<PriceChange> {
    Some(match now?.cmp(&then?) {
        Ordering::Equal => PriceChange::Same,
        Ordering::Greater => PriceChange::Up,
        Ordering::Less => PriceChange::Down,
    })
}

#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
mod tests;
