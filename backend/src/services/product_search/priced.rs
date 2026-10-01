//! A product priced for the quantity the household wants.

use rust_decimal::Decimal;

use crate::services::stores::deal::total_price;
use crate::services::stores::Product;

/// A product plus what it costs at the item's quantity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricedProduct {
    pub product: Product,
    /// The cost of the item's quantity, deals applied. `None` when the
    /// store shows no price.
    pub total_price: Option<Decimal>,
    /// A deal makes `total_price` lower than shelf price × quantity.
    pub deal_applied: bool,
    /// Why the unit price needs care, if it does — see
    /// [`super::comparability`].
    pub unit_price_note: Option<String>,
    /// Shelf price × quantity, kept to work out the deal's share.
    at_shelf: Option<Decimal>,
}

impl PricedProduct {
    /// Prices `product` for `quantity` units.
    pub fn new(product: Product, quantity: u32) -> Self {
        let total = product
            .price
            .map(|price| total_price(price, &product.deals, quantity));
        let at_shelf = product.price.map(|price| price * Decimal::from(quantity));
        Self {
            deal_applied: total.is_some() && total < at_shelf,
            total_price: total,
            unit_price_note: None,
            at_shelf,
            product,
        }
    }

    /// The unit price after deals: the shelf unit price scaled by the share
    /// of shelf cost the household actually pays. What results are ordered
    /// by, so a multibuy that is cheaper per litre sorts as cheaper.
    pub fn effective_unit_price(&self) -> Option<Decimal> {
        let unit = self.product.unit_price.as_ref()?.amount;
        match (self.total_price, self.at_shelf) {
            (Some(total), Some(at_shelf)) if !at_shelf.is_zero() => {
                Some((unit * total / at_shelf).round_dp(4))
            }
            _ => Some(unit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::product::fixtures::product;
    use crate::services::stores::{Deal, Store};
    use rust_decimal_macros::dec;

    #[test]
    fn prices_the_quantity_with_the_best_deal() {
        let mut cola = product(Store::Coles, "Cola", dec!(4.5));
        cola.deals.push(Deal {
            description: "2 for $5.80".into(),
            min_quantity: 2,
            unit_price: dec!(2.9),
        });

        let priced = PricedProduct::new(cola, 2);

        assert_eq!(priced.total_price, Some(dec!(5.8)));
        assert!(priced.deal_applied);
    }

    #[test]
    fn a_deal_lowers_the_effective_unit_price() {
        let mut cola = product(Store::Coles, "Cola", dec!(4));
        cola.unit_price = Some(crate::services::stores::UnitPrice {
            amount: dec!(0.32),
            per: crate::services::stores::Basis::Per100Millilitres,
            converted_from: None,
        });
        cola.deals.push(Deal {
            description: "2 for $6".into(),
            min_quantity: 2,
            unit_price: dec!(3),
        });

        assert_eq!(
            PricedProduct::new(cola.clone(), 1).effective_unit_price(),
            Some(dec!(0.32))
        );
        assert_eq!(
            PricedProduct::new(cola, 2).effective_unit_price(),
            Some(dec!(0.24))
        );
    }

    #[test]
    fn without_a_deal_it_is_shelf_price_times_quantity() {
        let priced = PricedProduct::new(product(Store::Coles, "Milk", dec!(4.95)), 2);
        assert_eq!(priced.total_price, Some(dec!(9.9)));
        assert!(!priced.deal_applied);
    }

    #[test]
    fn an_unpriced_product_has_no_total() {
        let mut gone = product(Store::Coles, "Gone", dec!(1));
        gone.price = None;
        assert_eq!(PricedProduct::new(gone, 3).total_price, None);
    }
}
