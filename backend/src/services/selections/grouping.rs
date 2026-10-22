//! Pairing each item's choices: the one the order buys, and the rest.
//!
//! Pure: takes the table's rows, returns one entry per item.

use std::collections::HashMap;

use uuid::Uuid;

use crate::models::db::ItemSelection;
use crate::services::stores::Store;

/// One item's choices.
#[derive(Debug, Clone)]
pub struct ItemChoices {
    /// The choice the order buys.
    pub for_order: ItemSelection,
    /// The item's choices at other stores, in the stores' display order.
    pub others: Vec<ItemSelection>,
}

/// Groups `rows` by item, in the order each item's `for_order` row arrives.
/// Rows of an item with no `for_order` row (the schema prevents it) are
/// dropped rather than guessed at.
pub fn by_item(rows: Vec<ItemSelection>) -> Vec<ItemChoices> {
    let mut others: HashMap<Uuid, Vec<ItemSelection>> = HashMap::new();
    let mut chosen = Vec::new();
    for row in rows {
        if row.for_order {
            chosen.push(row);
        } else {
            others.entry(row.grocery_item_id).or_default().push(row);
        }
    }
    chosen
        .into_iter()
        .map(|for_order| {
            let mut rest = others
                .remove(&for_order.grocery_item_id)
                .unwrap_or_default();
            rest.sort_by_key(|row| Store::ALL.iter().position(|store| *store == row.store));
            ItemChoices {
                for_order,
                others: rest,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::services::order_review::fixtures::{choice, item};

    fn not_for_order(mut row: ItemSelection) -> ItemSelection {
        row.for_order = false;
        row
    }

    #[test]
    fn an_item_with_one_choice_has_no_others() {
        let milk = item("milk", 1);
        let grouped = by_item(vec![choice(&milk, Store::Coles, "c-milk", dec!(4))]);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].for_order.product_id, "c-milk");
        assert!(grouped[0].others.is_empty());
    }

    #[test]
    fn the_other_stores_choice_sits_beside_the_one_to_buy() {
        let milk = item("milk", 1);
        let bread = item("bread", 1);
        let grouped = by_item(vec![
            not_for_order(choice(&milk, Store::Woolworths, "w-milk", dec!(3))),
            choice(&bread, Store::Woolworths, "w-bread", dec!(4)),
            choice(&milk, Store::Coles, "c-milk", dec!(4)),
        ]);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].for_order.product_id, "w-bread");
        assert_eq!(grouped[1].for_order.product_id, "c-milk");
        assert_eq!(grouped[1].others.len(), 1);
        assert_eq!(grouped[1].others[0].product_id, "w-milk");
    }

    #[test]
    fn rows_without_a_choice_to_buy_are_dropped() {
        let milk = item("milk", 1);
        let grouped = by_item(vec![not_for_order(choice(
            &milk,
            Store::Coles,
            "c",
            dec!(1),
        ))]);
        assert!(grouped.is_empty());
    }
}
