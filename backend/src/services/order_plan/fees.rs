//! What one store charges to deliver an order of a given size.
//!
//! Pure: the household's fee rules (Settings › Delivery) and a subtotal in,
//! the fee and whether the store would take the order out.

use rust_decimal::Decimal;

use crate::services::delivery_settings::FeeRules;

/// One store's delivery charge for an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryCharge {
    /// What delivery adds. Zero when it is free or the fee is not set.
    pub fee: Decimal,
    /// The household has set this store's fee; otherwise `fee` is a guess
    /// of zero and the screen says so.
    pub fee_known: bool,
    /// The order is big enough to deliver free.
    pub free: bool,
    /// The order is smaller than the store's minimum, so it cannot be placed.
    pub below_minimum: bool,
}

/// The charge for delivering `subtotal` worth of items under `rules`.
pub fn charge(rules: &FeeRules, subtotal: Decimal) -> DeliveryCharge {
    let free = rules
        .free_delivery_over
        .is_some_and(|threshold| subtotal >= threshold);
    let below_minimum = rules
        .minimum_order
        .is_some_and(|minimum| subtotal < minimum);
    let fee = match rules.delivery_fee {
        Some(_) if free => Decimal::ZERO,
        Some(fee) => fee,
        None => Decimal::ZERO,
    };
    DeliveryCharge {
        fee,
        fee_known: rules.delivery_fee.is_some(),
        free,
        below_minimum,
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::services::stores::Store;

    fn rules(
        fee: Option<Decimal>,
        free_over: Option<Decimal>,
        minimum: Option<Decimal>,
    ) -> FeeRules {
        FeeRules {
            store: Store::Woolworths,
            delivery_fee: fee,
            free_delivery_over: free_over,
            minimum_order: minimum,
        }
    }

    #[test]
    fn the_fee_is_charged_below_the_free_delivery_amount() {
        let charged = charge(&rules(Some(dec!(9)), Some(dec!(250)), None), dec!(249.99));
        assert_eq!(charged.fee, dec!(9));
        assert!(charged.fee_known);
        assert!(!charged.free);
    }

    #[test]
    fn delivery_is_free_from_the_free_delivery_amount() {
        let charged = charge(&rules(Some(dec!(9)), Some(dec!(250)), None), dec!(250));
        assert_eq!(charged.fee, dec!(0));
        assert!(charged.free);
    }

    #[test]
    fn an_unset_fee_counts_as_zero_and_says_so() {
        let charged = charge(&rules(None, None, None), dec!(40));
        assert_eq!(charged.fee, dec!(0));
        assert!(!charged.fee_known);
    }

    #[test]
    fn an_order_under_the_minimum_is_flagged() {
        let minimum = Some(dec!(50));
        assert!(charge(&rules(Some(dec!(9)), None, minimum), dec!(49.99)).below_minimum);
        assert!(!charge(&rules(Some(dec!(9)), None, minimum), dec!(50)).below_minimum);
    }
}
