//! One line per item per store, when an item has several products there.
//!
//! Pure. The planner compares stores item by item, so an item's products at
//! a store are joined into one line: their totals added up, or the first
//! line that cannot be bought (which makes the whole store unable to).

use rust_decimal::Decimal;

use crate::models::db::GroceryItem;
use crate::services::order_review::{LineStatus, OrderLine};

/// `lines` (an item's products at one store, at least one) as one line for
/// the whole `item`.
pub fn combine(item: &GroceryItem, lines: Vec<OrderLine>) -> Option<OrderLine> {
    let blocked = lines
        .iter()
        .position(|line| line.status != LineStatus::Priced);
    let total: Option<Decimal> = lines.iter().map(|line| line.total_price).sum();
    let deal_applied = lines.iter().any(|line| line.deal_applied);
    let mut lines = lines.into_iter();
    let mut joined = match blocked {
        Some(index) => lines.nth(index)?,
        None => {
            let first = lines.next()?;
            OrderLine {
                total_price: total,
                deal_applied,
                ..first
            }
        }
    };
    joined.item = item.clone();
    Some(joined)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::services::order_review::fixtures::{choice, item};
    use crate::services::stores::Store;

    fn line(
        item: &GroceryItem,
        product: &str,
        total: Option<Decimal>,
        status: LineStatus,
    ) -> OrderLine {
        OrderLine {
            item: item.clone(),
            choice: choice(item, Store::Coles, product, dec!(1)),
            status,
            price: Some(dec!(1)),
            total_price: total,
            deal_applied: false,
            price_change: None,
            problem: None,
        }
    }

    #[test]
    fn totals_add_up_for_the_whole_item() {
        let milk = item("milk", 3);
        let joined = combine(
            &milk,
            vec![
                line(&milk, "a", Some(dec!(4)), LineStatus::Priced),
                line(&milk, "b", Some(dec!(2.5)), LineStatus::Priced),
            ],
        )
        .unwrap();
        assert_eq!(joined.total_price, Some(dec!(6.5)));
        assert_eq!(joined.item.quantity, 3);
        assert_eq!(joined.choice.product_id, "a");
    }

    #[test]
    fn a_product_that_cannot_be_bought_blocks_the_store() {
        let milk = item("milk", 2);
        let joined = combine(
            &milk,
            vec![
                line(&milk, "a", Some(dec!(4)), LineStatus::Priced),
                line(&milk, "b", None, LineStatus::Unavailable),
            ],
        )
        .unwrap();
        assert_eq!(joined.status, LineStatus::Unavailable);
        assert_eq!(joined.choice.product_id, "b");
        assert_eq!(joined.counted_total(), None);
    }
}
