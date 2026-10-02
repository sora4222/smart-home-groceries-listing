//! What one product put in the trolley cost.
//!
//! The bookmarklet reports which products went in, not their prices. So the
//! price is the store's own answer at the moment of the fill — usually from
//! the 10-minute search cache — at the quantity put in the trolley, deals
//! applied. When the store cannot be asked, or no longer lists the product,
//! the price saved with the household's choice is used instead. Pure: no I/O.

use rust_decimal::{Decimal, RoundingStrategy};
use uuid::Uuid;

use crate::models::db::{GroceryItem, GroceryItemStatus, ItemSelection, TrolleyHandoffLine};
use crate::services::product_search::{PricedProduct, StoreOutcome};
use crate::services::stores::Store;

/// One product the household bought, priced, before its delivery share and
/// category are worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoughtLine {
    pub grocery_item_id: Uuid,
    pub item_name: String,
    pub item_status_before: GroceryItemStatus,
    pub store: Store,
    pub product_id: String,
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub shelf_price: Option<Decimal>,
    pub total_price: Decimal,
    pub store_category: Option<String>,
}

/// Prices `line` for `item`, from the store's fresh answer or else from the
/// saved `choice`. `None` when neither has a price for that product.
pub fn price_line(
    store: Store,
    line: &TrolleyHandoffLine,
    item: &GroceryItem,
    choice: Option<&ItemSelection>,
    outcome: Option<&StoreOutcome>,
) -> Option<BoughtLine> {
    let quantity = u32::try_from(line.quantity).ok().filter(|q| *q > 0)?;
    let base = BoughtLine {
        grocery_item_id: item.id,
        item_name: item.name.clone(),
        item_status_before: item.status,
        store,
        product_id: line.product_id.clone(),
        product_name: line.product_name.clone(),
        brand: None,
        package_size: None,
        quantity: line.quantity,
        unit_price: Decimal::ZERO,
        shelf_price: None,
        total_price: Decimal::ZERO,
        store_category: None,
    };
    from_store(&base, quantity, outcome).or_else(|| from_choice(&base, quantity, choice))
}

/// Priced from the store's answer, when it still lists the product with a price.
fn from_store(
    base: &BoughtLine,
    quantity: u32,
    outcome: Option<&StoreOutcome>,
) -> Option<BoughtLine> {
    let found = outcome?
        .products
        .iter()
        .find(|priced| priced.product.product_id == base.product_id)?;
    let product = found.product.clone();
    let total = PricedProduct::new(product.clone(), quantity).total_price?;
    Some(BoughtLine {
        product_name: product.name,
        brand: product.brand,
        package_size: product.package_size,
        unit_price: per_one(total, quantity),
        shelf_price: product.price,
        total_price: total,
        store_category: product.category,
        ..base.clone()
    })
}

/// Priced from the household's saved choice of this same product.
fn from_choice(
    base: &BoughtLine,
    quantity: u32,
    choice: Option<&ItemSelection>,
) -> Option<BoughtLine> {
    let choice = choice.filter(|c| c.store == base.store && c.product_id == base.product_id)?;
    let one = match (choice.total_price, u32::try_from(choice.priced_quantity)) {
        (Some(total), Ok(priced)) if priced > 0 => total / Decimal::from(priced),
        _ => choice.price?,
    };
    let total = money(one * Decimal::from(quantity));
    Some(BoughtLine {
        brand: choice.brand.clone(),
        package_size: choice.package_size.clone(),
        unit_price: per_one(total, quantity),
        shelf_price: choice.price,
        total_price: total,
        ..base.clone()
    })
}

/// What one cost, in cents.
fn per_one(total: Decimal, quantity: u32) -> Decimal {
    money(total / Decimal::from(quantity))
}

/// Rounds to whole cents.
fn money(amount: Decimal) -> Decimal {
    amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

#[cfg(test)]
mod tests;
