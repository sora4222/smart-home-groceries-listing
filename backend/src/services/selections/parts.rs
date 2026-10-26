//! An item split into one part per chosen product at a store, each part the
//! item with its quantity set to that product's count. Pure.
//!
//! The order review and the order planner price each part on its own, so a
//! product is priced (deals included) at the count it is bought at.

use super::allocation;
use crate::models::db::{GroceryItem, ItemSelection};

/// The parts of `item` for `rows`, its products at one store (oldest first).
/// A product with nothing to buy is left out.
pub fn parts(item: &GroceryItem, rows: Vec<ItemSelection>) -> Vec<(GroceryItem, ItemSelection)> {
    let stored: Vec<_> = rows.iter().map(|row| row.quantity).collect();
    rows.into_iter()
        .zip(allocation::allocate(&stored, item.quantity))
        .filter(|(_, count)| *count > 0)
        .map(|(row, count)| {
            let part = GroceryItem {
                quantity: count,
                ..item.clone()
            };
            (part, row)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::services::order_review::fixtures::{choice, item};
    use crate::services::stores::Store;

    #[test]
    fn each_product_is_a_part_at_its_count() {
        let milk = item("milk", 3);
        let mut a = choice(&milk, Store::Coles, "a", dec!(2));
        a.quantity = Some(2);
        let mut b = choice(&milk, Store::Coles, "b", dec!(2));
        b.quantity = Some(1);

        let split = parts(&milk, vec![a, b]);

        let counts: Vec<_> = split
            .iter()
            .map(|(part, row)| (row.product_id.as_str(), part.quantity))
            .collect();
        assert_eq!(counts, vec![("a", 2), ("b", 1)]);
        assert!(split.iter().all(|(part, _)| part.id == milk.id));
    }

    #[test]
    fn a_lone_product_is_the_whole_item() {
        let milk = item("milk", 4);
        let split = parts(&milk, vec![choice(&milk, Store::Coles, "a", dec!(2))]);
        assert_eq!(split[0].0.quantity, 4);
    }
}
