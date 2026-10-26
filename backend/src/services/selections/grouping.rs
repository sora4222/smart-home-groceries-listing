//! Grouping each item's choices: the products the order buys, with how many
//! of each, and the item's products at other stores.
//!
//! Pure: takes the table's rows and the items' quantities, returns one entry
//! per item.

use std::collections::HashMap;

use uuid::Uuid;

use super::allocation;
use crate::models::db::ItemSelection;
use crate::services::stores::Store;

/// A chosen product and how many of it to buy.
#[derive(Debug, Clone)]
pub struct Counted {
    pub selection: ItemSelection,
    pub quantity: i32,
}

/// One item's choices.
#[derive(Debug, Clone)]
pub struct ItemChoices {
    /// The oldest product the order buys.
    pub for_order: Counted,
    /// The order's other products for the item, at the same store.
    pub bought_with: Vec<Counted>,
    /// The item's products at other stores, in the stores' display order.
    pub others: Vec<Counted>,
}

/// Groups `rows` (oldest first) by item, in the order each item's first
/// `for_order` row arrives. `quantities` gives each item's quantity; an item
/// missing from it counts each product at the quantity it was priced for.
/// Rows of an item with nothing for the order (the app prevents it) are
/// dropped rather than guessed at.
pub fn by_item(rows: Vec<ItemSelection>, quantities: &HashMap<Uuid, i32>) -> Vec<ItemChoices> {
    let mut order: Vec<Uuid> = Vec::new();
    let mut bought: HashMap<Uuid, Vec<ItemSelection>> = HashMap::new();
    let mut others: HashMap<Uuid, Vec<ItemSelection>> = HashMap::new();
    for row in rows {
        let id = row.grocery_item_id;
        if row.for_order {
            if !bought.contains_key(&id) {
                order.push(id);
            }
            bought.entry(id).or_default().push(row);
        } else {
            others.entry(id).or_default().push(row);
        }
    }

    order
        .into_iter()
        .filter_map(|id| {
            let total = quantities.get(&id).copied();
            let mut products = counted(bought.remove(&id)?, total).into_iter();
            let for_order = products.next()?;
            let mut rest = others.remove(&id).unwrap_or_default();
            rest.sort_by_key(|row| Store::ALL.iter().position(|store| *store == row.store));
            let others = Store::ALL
                .iter()
                .flat_map(|store| {
                    let at: Vec<_> = rest.iter().filter(|r| r.store == *store).cloned().collect();
                    counted(at, total)
                })
                .collect();
            Some(ItemChoices {
                for_order,
                bought_with: products.collect(),
                others,
            })
        })
        .collect()
}

/// One store's products with their counts, adding up to `total`.
fn counted(rows: Vec<ItemSelection>, total: Option<i32>) -> Vec<Counted> {
    let Some(first) = rows.first() else {
        return Vec::new();
    };
    let total = total.unwrap_or(first.priced_quantity);
    allocation::counted(rows, |row| row.quantity, total)
        .into_iter()
        .map(|(selection, quantity)| Counted {
            selection,
            quantity,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::models::db::GroceryItem;
    use crate::services::order_review::fixtures::{choice, item};

    fn not_for_order(mut row: ItemSelection) -> ItemSelection {
        row.for_order = false;
        row
    }

    fn with_quantity(mut row: ItemSelection, quantity: i32) -> ItemSelection {
        row.quantity = Some(quantity);
        row
    }

    fn quantities(items: &[&GroceryItem]) -> HashMap<Uuid, i32> {
        items.iter().map(|i| (i.id, i.quantity)).collect()
    }

    #[test]
    fn an_item_with_one_choice_buys_its_whole_quantity() {
        let milk = item("milk", 3);
        let grouped = by_item(
            vec![choice(&milk, Store::Coles, "c-milk", dec!(4))],
            &quantities(&[&milk]),
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].for_order.selection.product_id, "c-milk");
        assert_eq!(grouped[0].for_order.quantity, 3);
        assert!(grouped[0].bought_with.is_empty());
        assert!(grouped[0].others.is_empty());
    }

    #[test]
    fn several_products_at_the_store_are_bought_together_with_counts() {
        let milk = item("milk", 3);
        let grouped = by_item(
            vec![
                with_quantity(choice(&milk, Store::Coles, "a", dec!(4)), 2),
                with_quantity(choice(&milk, Store::Coles, "b", dec!(4)), 1),
            ],
            &quantities(&[&milk]),
        );
        assert_eq!(grouped[0].for_order.quantity, 2);
        assert_eq!(grouped[0].bought_with.len(), 1);
        assert_eq!(grouped[0].bought_with[0].selection.product_id, "b");
        assert_eq!(grouped[0].bought_with[0].quantity, 1);
    }

    #[test]
    fn the_other_stores_choice_sits_beside_the_ones_to_buy() {
        let milk = item("milk", 1);
        let bread = item("bread", 1);
        let grouped = by_item(
            vec![
                not_for_order(choice(&milk, Store::Woolworths, "w-milk", dec!(3))),
                choice(&bread, Store::Woolworths, "w-bread", dec!(4)),
                choice(&milk, Store::Coles, "c-milk", dec!(4)),
            ],
            &quantities(&[&milk, &bread]),
        );
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].for_order.selection.product_id, "w-bread");
        assert_eq!(grouped[1].for_order.selection.product_id, "c-milk");
        assert_eq!(grouped[1].others.len(), 1);
        assert_eq!(grouped[1].others[0].selection.product_id, "w-milk");
    }

    #[test]
    fn rows_without_a_choice_to_buy_are_dropped() {
        let milk = item("milk", 1);
        let grouped = by_item(
            vec![not_for_order(choice(&milk, Store::Coles, "c", dec!(1)))],
            &quantities(&[&milk]),
        );
        assert!(grouped.is_empty());
    }
}
