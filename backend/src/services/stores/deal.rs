//! Multibuy deals and what they make an item cost.
//!
//! "2 for $6", "Pick any 2 for $5.80": each is a lower per-unit price that
//! applies to every complete group of `min_quantity`. Any remainder is paid
//! at the shelf price. The order optimiser uses [`total_price`], never the
//! shelf price alone.

use rust_decimal::Decimal;
use serde::Serialize;

/// A conditional price parsed from a store's product data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Deal {
    /// The store's own wording, e.g. "2 for $6.00".
    pub description: String,
    /// Units per group the deal price applies to.
    pub min_quantity: u32,
    /// Price per unit inside a complete group.
    pub unit_price: Decimal,
}

/// What `quantity` units cost at `shelf_price`, taking the best deal.
///
/// A deal that would cost more than the shelf price (stores do publish
/// those) is ignored, as is any deal with a zero group size.
pub fn total_price(shelf_price: Decimal, deals: &[Deal], quantity: u32) -> Decimal {
    let at_shelf = shelf_price * Decimal::from(quantity);
    deals
        .iter()
        .filter(|deal| deal.min_quantity > 0)
        .map(|deal| with_deal(shelf_price, deal, quantity))
        .fold(at_shelf, Decimal::min)
}

/// The cost of `quantity` units using one deal for every complete group.
fn with_deal(shelf_price: Decimal, deal: &Deal, quantity: u32) -> Decimal {
    let groups = quantity / deal.min_quantity;
    let remainder = quantity % deal.min_quantity;
    Decimal::from(groups * deal.min_quantity) * deal.unit_price
        + Decimal::from(remainder) * shelf_price
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn two_for_six() -> Deal {
        Deal {
            description: "2 for $6.00".into(),
            min_quantity: 2,
            unit_price: dec!(3),
        }
    }

    #[test]
    fn no_deal_is_shelf_price_times_quantity() {
        assert_eq!(total_price(dec!(5), &[], 3), dec!(15));
    }

    #[test]
    fn a_deal_needs_a_complete_group() {
        assert_eq!(total_price(dec!(5), &[two_for_six()], 1), dec!(5));
        assert_eq!(total_price(dec!(5), &[two_for_six()], 2), dec!(6));
    }

    #[test]
    fn the_remainder_pays_shelf_price() {
        assert_eq!(total_price(dec!(5), &[two_for_six()], 3), dec!(11));
    }

    #[test]
    fn the_cheapest_deal_wins_and_a_bad_deal_is_ignored() {
        let worse = Deal {
            description: "2 for $12".into(),
            min_quantity: 2,
            unit_price: dec!(6),
        };
        assert_eq!(
            total_price(dec!(5), std::slice::from_ref(&worse), 2),
            dec!(10)
        );
        assert_eq!(total_price(dec!(5), &[worse, two_for_six()], 4), dec!(12));
    }

    #[test]
    fn a_zero_sized_group_is_ignored() {
        let broken = Deal {
            description: "?".into(),
            min_quantity: 0,
            unit_price: dec!(0),
        };
        assert_eq!(total_price(dec!(5), &[broken], 2), dec!(10));
    }
}
