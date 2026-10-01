//! Normalising unit prices so products can be compared.
//!
//! Every price is restated against one basis per dimension: per 100 g, per
//! 100 mL, or per unit. When that needs a conversion (a price quoted per kg
//! restated per 100 g) the result remembers what it was calculated from, so
//! the web app can say "unit price calculated from 1kg".

use rust_decimal::Decimal;
use serde::Serialize;

use super::measure::{Dimension, Measure};

/// Decimal places a normalised unit price keeps. Per-sheet prices are
/// fractions of a cent, so this is wider than 2.
const UNIT_PRICE_PLACES: u32 = 4;

/// The common basis a unit price is stated against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Basis {
    #[serde(rename = "100g")]
    Per100Grams,
    #[serde(rename = "100mL")]
    Per100Millilitres,
    #[serde(rename = "unit")]
    PerUnit,
}

impl Basis {
    /// The basis for a dimension.
    pub fn for_dimension(dimension: Dimension) -> Self {
        match dimension {
            Dimension::Mass => Basis::Per100Grams,
            Dimension::Volume => Basis::Per100Millilitres,
            Dimension::Count => Basis::PerUnit,
        }
    }

    /// How many base units (g, mL, items) the basis covers.
    fn base_amount(self) -> Decimal {
        match self {
            Basis::Per100Grams | Basis::Per100Millilitres => Decimal::ONE_HUNDRED,
            Basis::PerUnit => Decimal::ONE,
        }
    }
}

/// A unit price restated against a common [`Basis`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnitPrice {
    /// Price per basis, e.g. `0.49` per 100 g.
    pub amount: Decimal,
    pub per: Basis,
    /// The store's own measure, present only when converting from it changed
    /// the basis — e.g. `1kg` for a price the store quoted per kilogram.
    pub converted_from: Option<String>,
}

/// Restates `price` for `measure` against the measure's common basis.
pub fn normalise(price: Decimal, measure: &Measure) -> UnitPrice {
    let per = Basis::for_dimension(measure.dimension);
    let basis_amount = per.base_amount();
    let amount = (price * basis_amount / measure.base_amount)
        .round_dp(UNIT_PRICE_PLACES)
        .normalize();
    let converted_from = (measure.base_amount != basis_amount).then(|| measure.original.clone());
    UnitPrice {
        amount,
        per,
        converted_from,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::measure::parse;
    use rust_decimal_macros::dec;

    #[test]
    fn a_kilogram_price_becomes_per_100_grams_and_says_so() {
        let unit = normalise(dec!(4.90), &parse("1kg").unwrap());
        assert_eq!(unit.amount, dec!(0.49));
        assert_eq!(unit.per, Basis::Per100Grams);
        assert_eq!(unit.converted_from.as_deref(), Some("1kg"));
    }

    #[test]
    fn a_price_already_on_the_basis_is_not_marked_converted() {
        let unit = normalise(dec!(3.13), &parse("100G").unwrap());
        assert_eq!(unit.amount, dec!(3.13));
        assert_eq!(unit.converted_from, None);
    }

    #[test]
    fn litres_become_per_100_millilitres() {
        let unit = normalise(dec!(1.65), &parse("1L").unwrap());
        assert_eq!(unit.amount, dec!(0.165));
        assert_eq!(unit.per, Basis::Per100Millilitres);
    }

    #[test]
    fn sheet_counts_become_per_unit() {
        let unit = normalise(dec!(0.28), &parse("100 sheets").unwrap());
        assert_eq!(unit.amount, dec!(0.0028));
        assert_eq!(unit.per, Basis::PerUnit);
        assert_eq!(unit.converted_from.as_deref(), Some("100 sheets"));
    }

    #[test]
    fn serialises_the_amount_as_an_exact_string() {
        let unit = normalise(dec!(4.90), &parse("1kg").unwrap());
        let json = serde_json::to_value(unit).unwrap();
        assert_eq!(json["amount"], "0.49");
        assert_eq!(json["per"], "100g");
    }
}
