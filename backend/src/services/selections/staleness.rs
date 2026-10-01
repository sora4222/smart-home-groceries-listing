//! When an edit to a list item makes its chosen product stale.
//!
//! A product is chosen for what the item *is*: its name and its chips. Change
//! either and the product may no longer fit ("milk" renamed to "oat milk", or
//! a "3 ply" chip added after a 2-ply roll was chosen), so the choice is
//! forgotten. Quantity and note do not change what the item is, so the
//! choice stays; the order screen re-prices for the new quantity.

use crate::models::db::GroceryItem;
use crate::services::grocery::repository::normalise;

/// Whether a choice made for `before` still fits `after`.
///
/// Names are compared the way duplicate detection compares them, so
/// "Milk" → "milk " is not a rename.
pub fn choice_still_fits(before: &GroceryItem, after: &GroceryItem) -> bool {
    normalise(&before.name) == normalise(&after.name) && before.filter_terms == after.filter_terms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::db::{GroceryItemSource, GroceryItemStatus};

    fn item(name: &str, terms: &[&str], quantity: i32) -> GroceryItem {
        GroceryItem {
            id: uuid::Uuid::nil(),
            name: name.into(),
            quantity,
            status: GroceryItemStatus::Active,
            source: GroceryItemSource::Manual,
            note: None,
            filter_terms: terms.iter().map(|t| t.to_string()).collect(),
            added_by_user_id: None,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn a_new_quantity_keeps_the_choice() {
        assert!(choice_still_fits(
            &item("milk", &[], 1),
            &item("milk", &[], 3)
        ));
    }

    #[test]
    fn a_change_of_case_or_spacing_keeps_the_choice() {
        assert!(choice_still_fits(
            &item("full cream milk", &[], 1),
            &item(" Full  Cream Milk", &[], 1)
        ));
    }

    #[test]
    fn a_rename_makes_it_stale() {
        assert!(!choice_still_fits(
            &item("milk", &[], 1),
            &item("oat milk", &[], 1)
        ));
    }

    #[test]
    fn a_change_of_chips_makes_it_stale() {
        assert!(!choice_still_fits(
            &item("toilet paper", &[], 1),
            &item("toilet paper", &["3 ply"], 1)
        ));
    }
}
