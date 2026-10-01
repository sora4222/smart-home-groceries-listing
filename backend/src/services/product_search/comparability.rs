//! Saying when a unit price cannot be compared at face value.
//!
//! Most results for an item share a basis (milk: per 100 mL). A product on
//! a different basis ("1 pack" beside "200 g"), or with no unit price at
//! all, is flagged rather than silently ranked, so the household can decide.
//! A converted price is labelled with what it was converted from.

use std::collections::HashMap;

use super::priced::PricedProduct;
use crate::services::stores::Basis;

/// Note for a product whose unit price is on another basis than the rest.
pub const UNITS_DIFFER: &str = "units differ — compare by hand";
/// Note for a product the store gave no unit price for.
pub const NO_UNIT_PRICE: &str = "no unit price from the store";

/// The basis most of the products are priced on, across every store, so
/// Woolworths and Coles are judged against the same one. A tie goes to
/// the basis that appears first.
pub fn common_basis(products: &[PricedProduct]) -> Option<Basis> {
    let mut counts: HashMap<Basis, (usize, usize)> = HashMap::new();
    for (position, unit) in products
        .iter()
        .filter_map(|p| p.product.unit_price.as_ref())
        .enumerate()
    {
        counts.entry(unit.per).or_insert((0, position)).0 += 1;
    }
    counts
        .into_iter()
        .max_by(|(_, (a_count, a_first)), (_, (b_count, b_first))| {
            a_count.cmp(b_count).then(b_first.cmp(a_first))
        })
        .map(|(basis, _)| basis)
}

/// Sets each product's `unit_price_note` against `common`.
pub fn annotate(products: &mut [PricedProduct], common: Option<Basis>) {
    for priced in products {
        priced.unit_price_note = note(priced, common);
    }
}

/// The note one product gets, if any.
fn note(priced: &PricedProduct, common: Option<Basis>) -> Option<String> {
    let Some(unit) = priced.product.unit_price.as_ref() else {
        return Some(NO_UNIT_PRICE.to_string());
    };
    if common.is_some_and(|basis| basis != unit.per) {
        return Some(UNITS_DIFFER.to_string());
    }
    unit.converted_from
        .as_ref()
        .map(|original| format!("unit price calculated from {original}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::product::fixtures::product;
    use crate::services::stores::{Store, UnitPrice};
    use rust_decimal_macros::dec;

    fn priced(per: Option<Basis>, converted_from: Option<&str>) -> PricedProduct {
        let mut p = product(Store::Woolworths, "x", dec!(1));
        p.unit_price = per.map(|per| UnitPrice {
            amount: dec!(1),
            per,
            converted_from: converted_from.map(str::to_string),
        });
        PricedProduct::new(p, 1)
    }

    #[test]
    fn the_common_basis_is_the_most_frequent() {
        let products = vec![
            priced(Some(Basis::PerUnit), None),
            priced(Some(Basis::Per100Grams), None),
            priced(Some(Basis::Per100Grams), None),
            priced(None, None),
        ];
        assert_eq!(common_basis(&products), Some(Basis::Per100Grams));
        assert_eq!(common_basis(&[]), None);
    }

    #[test]
    fn notes_say_what_needs_care() {
        let mut products = vec![
            priced(Some(Basis::Per100Grams), None),
            priced(Some(Basis::Per100Grams), Some("1kg")),
            priced(Some(Basis::PerUnit), None),
            priced(None, None),
        ];
        annotate(&mut products, Some(Basis::Per100Grams));
        let notes: Vec<Option<&str>> = products
            .iter()
            .map(|p| p.unit_price_note.as_deref())
            .collect();
        assert_eq!(
            notes,
            [
                None,
                Some("unit price calculated from 1kg"),
                Some(UNITS_DIFFER),
                Some(NO_UNIT_PRICE)
            ]
        );
    }
}
