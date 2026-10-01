//! Gathering re-priced lines into the order the household reviews.
//!
//! Lines are grouped by store, in the stores' display order, each group with
//! its subtotal. Items with no product chosen are listed apart so the screen
//! can send the household back to choose one. Pure: no I/O.

use rust_decimal::Decimal;

use super::line::OrderLine;
use crate::models::db::GroceryItem;
use crate::services::stores::Store;

/// Everything bought at one store.
#[derive(Debug, Clone)]
pub struct StoreOrder {
    pub store: Store,
    pub lines: Vec<OrderLine>,
    /// The sum of every line with a price today.
    pub subtotal: Decimal,
    /// Every line has a price today, so the subtotal is the whole cost.
    pub complete: bool,
}

/// The order as it stands right now.
#[derive(Debug, Clone)]
pub struct OrderReview {
    /// Only stores with at least one line, Woolworths first.
    pub stores: Vec<StoreOrder>,
    /// Committed items with no product chosen.
    pub unchosen: Vec<GroceryItem>,
    /// The sum of the store subtotals. Delivery fees are not included.
    pub total: Decimal,
    /// Every store is complete and every item has a product.
    pub complete: bool,
}

impl OrderReview {
    /// Lines that cannot be bought as they are, across every store.
    pub fn problem_count(&self) -> usize {
        self.stores
            .iter()
            .flat_map(|store| &store.lines)
            .filter(|line| line.problem.is_some())
            .count()
    }
}

/// Builds the order from re-priced `lines` and the `unchosen` items.
/// Lines keep the order they arrive in within their store.
pub fn build(lines: Vec<OrderLine>, unchosen: Vec<GroceryItem>) -> OrderReview {
    let stores: Vec<StoreOrder> = Store::ALL
        .into_iter()
        .filter_map(|store| store_order(store, &lines))
        .collect();
    let total = stores.iter().map(|s| s.subtotal).sum();
    let complete = unchosen.is_empty() && stores.iter().all(|s| s.complete);
    OrderReview {
        stores,
        unchosen,
        total,
        complete,
    }
}

/// `store`'s part of the order, or `None` when nothing is bought there.
fn store_order(store: Store, lines: &[OrderLine]) -> Option<StoreOrder> {
    let lines: Vec<OrderLine> = lines
        .iter()
        .filter(|line| line.choice.store == store)
        .cloned()
        .collect();
    if lines.is_empty() {
        return None;
    }
    let subtotal = lines.iter().filter_map(OrderLine::counted_total).sum();
    let complete = lines.iter().all(|line| line.counted_total().is_some());
    Some(StoreOrder {
        store,
        lines,
        subtotal,
        complete,
    })
}

#[cfg(test)]
mod tests {
    use super::super::line::fixtures::{choice, item};
    use super::super::line::{LineStatus, OrderLine};
    use super::*;
    use rust_decimal_macros::dec;

    fn line(name: &str, store: Store, total: Option<Decimal>) -> OrderLine {
        let item = item(name, 1);
        let choice = choice(&item, store, name, dec!(1));
        OrderLine {
            item,
            choice,
            status: if total.is_some() {
                LineStatus::Priced
            } else {
                LineStatus::NotOffered
            },
            price: total,
            total_price: total,
            deal_applied: false,
            price_change: None,
            problem: total.is_none().then(|| "gone".to_string()),
        }
    }

    #[test]
    fn groups_lines_by_store_woolworths_first_with_subtotals() {
        let review = build(
            vec![
                line("milk", Store::Coles, Some(dec!(4.95))),
                line("bread", Store::Woolworths, Some(dec!(3.50))),
                line("eggs", Store::Coles, Some(dec!(6.00))),
            ],
            Vec::new(),
        );

        let stores: Vec<Store> = review.stores.iter().map(|s| s.store).collect();
        assert_eq!(stores, vec![Store::Woolworths, Store::Coles]);
        assert_eq!(review.stores[1].subtotal, dec!(10.95));
        assert_eq!(review.stores[1].lines[0].item.name, "milk");
        assert_eq!(review.total, dec!(14.45));
        assert!(review.complete);
    }

    #[test]
    fn a_store_with_nothing_bought_is_left_out() {
        let review = build(vec![line("milk", Store::Coles, Some(dec!(1)))], Vec::new());
        assert_eq!(review.stores.len(), 1);
        assert_eq!(review.stores[0].store, Store::Coles);
    }

    #[test]
    fn a_line_without_a_price_is_not_counted_and_marks_the_store_incomplete() {
        let review = build(
            vec![
                line("milk", Store::Coles, Some(dec!(4.95))),
                line("gone", Store::Coles, None),
            ],
            Vec::new(),
        );

        assert_eq!(review.stores[0].subtotal, dec!(4.95));
        assert!(!review.stores[0].complete);
        assert!(!review.complete);
        assert_eq!(review.problem_count(), 1);
    }

    #[test]
    fn an_item_with_no_product_makes_the_order_incomplete() {
        let review = build(
            vec![line("milk", Store::Coles, Some(dec!(1)))],
            vec![item("bread", 1)],
        );
        assert!(review.stores[0].complete);
        assert!(!review.complete);
        assert_eq!(review.unchosen[0].name, "bread");
    }

    #[test]
    fn an_empty_order_costs_nothing() {
        let review = build(Vec::new(), Vec::new());
        assert!(review.stores.is_empty());
        assert_eq!(review.total, Decimal::ZERO);
        assert!(review.complete);
    }
}
