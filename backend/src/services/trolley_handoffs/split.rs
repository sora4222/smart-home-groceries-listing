//! Turning list items' chosen products into handoff lines, one per product,
//! each with how many of it to add.
//!
//! Pure. An item with several products at the store gets a line for each,
//! counted by [`allocation::allocate`]; a product with nothing to buy (more
//! products than the item wants) gets no line.

use uuid::Uuid;

use super::line_repository::ChosenLine;
use crate::services::selections::allocation;

/// One chosen product as read for a handoff: `(grocery_item_id, product_id,
/// product_name, item_quantity, stored_quantity)`, each item's products
/// together and oldest first.
pub type ChosenRow = (Uuid, String, String, i32, Option<i32>);

/// The handoff lines for `rows`, in the order given.
pub fn into_lines(rows: Vec<ChosenRow>) -> Vec<ChosenLine> {
    let mut lines = Vec::new();
    let mut rest = rows.as_slice();
    while let Some(first) = rest.first() {
        let len = rest.iter().take_while(|row| row.0 == first.0).count();
        let (item, after) = rest.split_at(len);
        let stored: Vec<_> = item.iter().map(|row| row.4).collect();
        for (row, count) in item.iter().zip(allocation::allocate(&stored, first.3)) {
            if count > 0 {
                lines.push((row.0, row.1.clone(), row.2.clone(), count));
            }
        }
        rest = after;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(item: Uuid, product: &str, total: i32, stored: Option<i32>) -> ChosenRow {
        (item, product.into(), product.into(), total, stored)
    }

    #[test]
    fn a_lone_product_adds_the_item_quantity() {
        let milk = Uuid::new_v4();
        let lines = into_lines(vec![row(milk, "a", 3, None)]);
        assert_eq!(lines, vec![(milk, "a".into(), "a".into(), 3)]);
    }

    #[test]
    fn several_products_split_the_quantity() {
        let milk = Uuid::new_v4();
        let bread = Uuid::new_v4();
        let lines = into_lines(vec![
            row(milk, "a", 3, Some(2)),
            row(milk, "b", 3, Some(1)),
            row(bread, "c", 1, None),
        ]);
        let counts: Vec<_> = lines.iter().map(|l| (l.1.as_str(), l.3)).collect();
        assert_eq!(counts, vec![("a", 2), ("b", 1), ("c", 1)]);
    }

    #[test]
    fn a_product_with_nothing_to_buy_gets_no_line() {
        let milk = Uuid::new_v4();
        let lines = into_lines(vec![row(milk, "a", 1, Some(1)), row(milk, "b", 1, Some(1))]);
        assert_eq!(lines.len(), 1);
    }
}
