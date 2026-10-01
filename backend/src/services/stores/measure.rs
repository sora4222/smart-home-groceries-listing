//! Reading the measure a store quotes a unit price against.
//!
//! Woolworths writes `100G`, `1KG`, `1L` or `100 sheets`; Coles writes
//! `$4.90/ 1kg` or `$0.31/ 100ea`. Both reduce to an amount of a base unit:
//! grams, millilitres, or a count of things.

use rust_decimal::Decimal;

/// What a measure counts in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    /// Grams.
    Mass,
    /// Millilitres.
    Volume,
    /// Sheets, pieces, each — anything counted rather than weighed.
    Count,
}

/// A quantity of a base unit, e.g. `1kg` is 1000 g of [`Dimension::Mass`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measure {
    /// How many base units (g, mL, items) the quoted price covers.
    pub base_amount: Decimal,
    pub dimension: Dimension,
    /// The measure as the store wrote it, tidied: `1kg`, `100 sheets`.
    pub original: String,
}

/// Parses a measure such as `100G`, `1KG`, `1.25L`, `100 sheets` or `ea`.
///
/// `None` when the unit is not one this application knows how to compare —
/// the caller then shows the product without a unit price rather than
/// guessing.
pub fn parse(raw: &str) -> Option<Measure> {
    let text = raw.trim();
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let amount: Decimal = if number.is_empty() {
        Decimal::ONE
    } else {
        number.parse().ok()?
    };
    if amount <= Decimal::ZERO {
        return None;
    }
    let unit = unit.trim().to_ascii_lowercase();
    let (dimension, factor) = unit_factor(&unit)?;
    Some(Measure {
        base_amount: amount * factor,
        dimension,
        original: tidy(number, &unit),
    })
}

/// Splits a comparable-price string such as `$4.90/ 1kg` or `$3.13 / 100G`
/// into its price and measure.
pub fn parse_comparable(raw: &str) -> Option<(Decimal, Measure)> {
    let (price, measure) = raw.split_once('/')?;
    let price: Decimal = price.trim().trim_start_matches('$').trim().parse().ok()?;
    Some((price, parse(measure)?))
}

/// The dimension a unit word belongs to and how many base units one of it is.
fn unit_factor(unit: &str) -> Option<(Dimension, Decimal)> {
    let thousand = Decimal::from(1000);
    match unit {
        "g" | "gm" | "gram" | "grams" => Some((Dimension::Mass, Decimal::ONE)),
        "kg" | "kilogram" | "kilograms" => Some((Dimension::Mass, thousand)),
        "ml" | "millilitre" | "millilitres" => Some((Dimension::Volume, Decimal::ONE)),
        "l" | "lt" | "litre" | "litres" => Some((Dimension::Volume, thousand)),
        "ea" | "each" | "sheet" | "sheets" | "unit" | "units" | "pack" | "pk" => {
            Some((Dimension::Count, Decimal::ONE))
        }
        _ => None,
    }
}

/// `100` + `sheets` → `100 sheets`; `1` + `kg` → `1kg`.
fn tidy(number: &str, unit: &str) -> String {
    let number = if number.is_empty() { "1" } else { number };
    if unit.len() > 2 {
        format!("{number} {unit}")
    } else {
        format!("{number}{unit}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn reads_woolworths_measures() {
        let kg = parse("1KG").unwrap();
        assert_eq!(kg.base_amount, dec!(1000));
        assert_eq!(kg.dimension, Dimension::Mass);
        assert_eq!(kg.original, "1kg");

        assert_eq!(parse("100ML").unwrap().dimension, Dimension::Volume);
        assert_eq!(parse("1.25L").unwrap().base_amount, dec!(1250));

        let sheets = parse("100 sheets").unwrap();
        assert_eq!(sheets.dimension, Dimension::Count);
        assert_eq!(sheets.original, "100 sheets");
    }

    #[test]
    fn a_bare_unit_means_one() {
        assert_eq!(parse("ea").unwrap().base_amount, Decimal::ONE);
    }

    #[test]
    fn unknown_units_are_not_guessed() {
        assert_eq!(parse("1 bunch"), None);
        assert_eq!(parse("0g"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn reads_coles_comparable_strings() {
        let (price, measure) = parse_comparable("$4.90/ 1kg").unwrap();
        assert_eq!(price, dec!(4.90));
        assert_eq!(measure.base_amount, dec!(1000));

        let (price, measure) = parse_comparable("$3.13 / 100G").unwrap();
        assert_eq!(price, dec!(3.13));
        assert_eq!(measure.base_amount, dec!(100));
    }

    #[test]
    fn rejects_comparable_strings_without_a_price() {
        assert_eq!(parse_comparable(""), None);
        assert_eq!(parse_comparable("each"), None);
    }
}
