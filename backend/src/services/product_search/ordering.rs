//! The order a store's results are shown in.
//!
//! Cheapest first, judged the way the household pays: by unit price after
//! deals at the item's quantity. Products that cannot be bought go last, and
//! products whose unit price cannot be compared follow the comparable ones,
//! ordered by what the item's quantity would cost. Nothing is promoted for
//! any other reason — the store's own "relevance" order (which includes paid
//! placement) is only the final tie-break.

use std::cmp::Ordering;

use rust_decimal::Decimal;

use super::priced::PricedProduct;
use crate::services::stores::Basis;

/// Sorts `products` in place. The sort is stable, so equal products keep the
/// store's order.
pub fn sort(products: &mut [PricedProduct], common: Option<Basis>) {
    products.sort_by(|a, b| compare(a, b, common));
}

/// Available before unavailable, comparable before not, then cheapest.
fn compare(a: &PricedProduct, b: &PricedProduct, common: Option<Basis>) -> Ordering {
    let rank = |p: &PricedProduct| (!p.product.available, comparable_price(p, common).is_none());
    rank(a)
        .cmp(&rank(b))
        .then_with(|| cheaper(comparable_price(a, common), comparable_price(b, common)))
        .then_with(|| cheaper(a.total_price, b.total_price))
}

/// The effective unit price, when it is on the common basis.
fn comparable_price(p: &PricedProduct, common: Option<Basis>) -> Option<Decimal> {
    let unit = p.product.unit_price.as_ref()?;
    if common.is_some_and(|basis| basis != unit.per) {
        return None;
    }
    p.effective_unit_price()
}

/// Lower first; a missing value sorts after any present one.
fn cheaper(a: Option<Decimal>, b: Option<Decimal>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::product::fixtures::product;
    use crate::services::stores::{Store, UnitPrice};
    use rust_decimal_macros::dec;

    fn priced(name: &str, price: Decimal, unit: Option<(Decimal, Basis)>) -> PricedProduct {
        let mut p = product(Store::Coles, name, price);
        p.unit_price = unit.map(|(amount, per)| UnitPrice {
            amount,
            per,
            converted_from: None,
        });
        PricedProduct::new(p, 1)
    }

    fn names(products: &[PricedProduct]) -> Vec<&str> {
        products.iter().map(|p| p.product.name.as_str()).collect()
    }

    #[test]
    fn cheapest_per_unit_first_then_the_rest() {
        let ml = Basis::Per100Millilitres;
        let mut products = vec![
            priced("small dear", dec!(2), Some((dec!(0.40), ml))),
            priced("pack", dec!(1), Some((dec!(0.10), Basis::PerUnit))),
            priced("big cheap", dec!(5), Some((dec!(0.20), ml))),
            priced("no unit", dec!(3), None),
        ];
        let mut gone = priced("gone", dec!(1), Some((dec!(0.01), ml)));
        gone.product.available = false;
        products.push(gone);

        sort(&mut products, Some(ml));

        assert_eq!(
            names(&products),
            ["big cheap", "small dear", "pack", "no unit", "gone"]
        );
    }
}
