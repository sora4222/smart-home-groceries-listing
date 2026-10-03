//! A product's past purchases, for the product picker.
//!
//! When a household member picks a product, the picker shows how many times
//! it was bought and what it cost each time. Grouping is pure; the read is in
//! [`super::read_repository::of_products`].

use crate::models::db::Purchase;
use crate::services::stores::Store;

/// Every purchase of one product at one store, newest first.
#[derive(Debug, Clone)]
pub struct ProductHistory {
    pub store: Store,
    pub product_id: String,
    pub purchases: Vec<Purchase>,
}

impl ProductHistory {
    /// How many times it was bought (one per shop it was in).
    pub fn times_bought(&self) -> usize {
        self.purchases.len()
    }
}

/// Groups `purchases` (newest first) by product, keeping that order inside
/// each product. Products appear in the order of their newest purchase.
pub fn group(purchases: Vec<Purchase>) -> Vec<ProductHistory> {
    let mut grouped: Vec<ProductHistory> = Vec::new();
    for purchase in purchases {
        let existing = grouped
            .iter_mut()
            .find(|h| h.store == purchase.store && h.product_id == purchase.product_id);
        match existing {
            Some(history) => history.purchases.push(purchase),
            None => grouped.push(ProductHistory {
                store: purchase.store,
                product_id: purchase.product_id.clone(),
                purchases: vec![purchase],
            }),
        }
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::purchases::fixtures::purchase;
    use rust_decimal_macros::dec;

    #[test]
    fn purchases_are_grouped_by_store_and_product() {
        let grouped = group(vec![
            purchase(Store::Coles, "c-milk", dec!(4.95), "2026-09-20"),
            purchase(Store::Woolworths, "c-milk", dec!(3.10), "2026-09-15"),
            purchase(Store::Coles, "c-milk", dec!(4.50), "2026-09-01"),
        ]);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].store, Store::Coles);
        assert_eq!(grouped[0].times_bought(), 2);
        let prices: Vec<_> = grouped[0].purchases.iter().map(|p| p.unit_price).collect();
        assert_eq!(prices, vec![dec!(4.95), dec!(4.50)]);
        assert_eq!(grouped[1].store, Store::Woolworths);
    }

    #[test]
    fn nothing_bought_is_no_history() {
        assert!(group(Vec::new()).is_empty());
    }
}
