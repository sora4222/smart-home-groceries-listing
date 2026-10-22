//! Which of two ways to buy the order is better, under the household's mode.
//!
//! Pure. Every mode first prefers an option that buys every item, then one
//! the stores would take and that keeps within the delivery cap; then:
//!
//! | Mode | Then by |
//! |---|---|
//! | minimise total, single store, manual | total, then delivery |
//! | minimise delivery | delivery, then total |
//!
//! Fewer stores breaks a tie (one delivery to wait for). Single-store and
//! manual modes put their own option first ([`preferred_kind`]); the rest are
//! still ranked, so the household can see what the choice costs.

use std::cmp::Ordering;

use rust_decimal::Decimal;

use super::option::{OptionKind, PlanOption};
use crate::models::delivery_rows::OrderMode;
use crate::services::stores::Store;

/// The numbers two options are compared by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    pub complete: bool,
    pub acceptable: bool,
    pub total: Decimal,
    pub delivery: Decimal,
    pub stores_used: usize,
}

impl From<&PlanOption> for Measure {
    fn from(option: &PlanOption) -> Self {
        Self {
            complete: option.complete(),
            acceptable: option.acceptable(),
            total: option.total,
            delivery: option.delivery_total,
            stores_used: option.parts.len(),
        }
    }
}

/// `Less` when `a` is the better option under `mode`.
pub fn compare(mode: OrderMode, a: &Measure, b: &Measure) -> Ordering {
    let first = b
        .complete
        .cmp(&a.complete)
        .then(b.acceptable.cmp(&a.acceptable));
    let money = match mode {
        OrderMode::MinimiseDelivery => a.delivery.cmp(&b.delivery).then(a.total.cmp(&b.total)),
        _ => a.total.cmp(&b.total).then(a.delivery.cmp(&b.delivery)),
    };
    first.then(money).then(a.stores_used.cmp(&b.stores_used))
}

/// The option a mode puts first whatever it costs, if any.
pub fn preferred_kind(mode: OrderMode) -> Option<OptionKind> {
    match mode {
        OrderMode::WoolworthsOnly => Some(OptionKind::Only(Store::Woolworths)),
        OrderMode::ColesOnly => Some(OptionKind::Only(Store::Coles)),
        OrderMode::Manual => Some(OptionKind::Current),
        OrderMode::MinimiseTotal | OrderMode::MinimiseDelivery => None,
    }
}

/// Sorts `options` best first under `mode`.
pub fn sort(mode: OrderMode, options: &mut [PlanOption]) {
    let preferred = preferred_kind(mode);
    options.sort_by(|a, b| {
        let a_first = Some(a.kind) == preferred;
        let b_first = Some(b.kind) == preferred;
        b_first
            .cmp(&a_first)
            .then_with(|| compare(mode, &Measure::from(&*a), &Measure::from(&*b)))
    });
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    fn measure(complete: bool, acceptable: bool, total: Decimal, delivery: Decimal) -> Measure {
        Measure {
            complete,
            acceptable,
            total,
            delivery,
            stores_used: 1,
        }
    }

    #[test]
    fn a_complete_order_beats_a_cheaper_incomplete_one() {
        let complete = measure(true, true, dec!(50), dec!(9));
        let partial = measure(false, true, dec!(20), dec!(0));
        assert_eq!(
            compare(OrderMode::MinimiseTotal, &complete, &partial),
            Ordering::Less
        );
    }

    #[test]
    fn an_order_the_store_takes_beats_a_cheaper_one_it_refuses() {
        let fine = measure(true, true, dec!(50), dec!(9));
        let refused = measure(true, false, dec!(40), dec!(0));
        assert_eq!(
            compare(OrderMode::MinimiseTotal, &fine, &refused),
            Ordering::Less
        );
    }

    #[test]
    fn minimise_total_looks_at_the_total_first() {
        let cheap_total = measure(true, true, dec!(50), dec!(15));
        let cheap_delivery = measure(true, true, dec!(55), dec!(0));
        assert_eq!(
            compare(OrderMode::MinimiseTotal, &cheap_total, &cheap_delivery),
            Ordering::Less
        );
        assert_eq!(
            compare(OrderMode::MinimiseDelivery, &cheap_delivery, &cheap_total),
            Ordering::Less
        );
    }

    #[test]
    fn fewer_stores_break_a_tie() {
        let one = measure(true, true, dec!(50), dec!(9));
        let two = Measure {
            stores_used: 2,
            ..one
        };
        assert_eq!(
            compare(OrderMode::MinimiseTotal, &one, &two),
            Ordering::Less
        );
    }
}
