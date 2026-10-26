//! Turning list items' chosen products into handoff lines, one per product,
//! each with how many of it to add.
//!
//! Pure. An item with several products at the store gets a line for each,
//! counted by [`allocation::allocate`]; a product with nothing to buy (more
//! products than the item wants) gets no line.

use uuid::Uuid;

use super::line_repository::ChosenLine;
use crate::services::selections::allocation;

/// One chosen product as read for a handoff, each item's products together
/// and oldest first.
#[derive(Debug, sqlx::FromRow)]
pub struct ChosenRow {
    pub grocery_item_id: Uuid,
    pub product_id: String,
    pub product_name: String,
    pub item_quantity: i32,
    pub stored_quantity: Option<i32>,
}

/// The handoff lines for `rows`, in the order given.
pub fn into_lines(rows: Vec<ChosenRow>) -> Vec<ChosenLine> {
    rows.chunk_by(|a, b| a.grocery_item_id == b.grocery_item_id)
        .flat_map(|item| {
            let total = item[0].item_quantity;
            allocation::counted(item.iter().collect(), |row| row.stored_quantity, total)
        })
        .filter(|(_, count)| *count > 0)
        .map(|(row, count)| {
            (
                row.grocery_item_id,
                row.product_id.clone(),
                row.product_name.clone(),
                count,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(item: Uuid, product: &str, total: i32, stored: Option<i32>) -> ChosenRow {
        ChosenRow {
            grocery_item_id: item,
            product_id: product.into(),
            product_name: product.into(),
            item_quantity: total,
            stored_quantity: stored,
        }
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
