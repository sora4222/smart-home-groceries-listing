//! Turning the stores' JSON numbers into exact decimals.
//!
//! Both stores send prices as JSON floats (`5.95`). Money is summed across a
//! whole order, so it is held as [`Decimal`] from the moment it arrives;
//! binary floating point cannot represent a cent exactly.

use rust_decimal::Decimal;

/// Decimal places kept from a store's number. Unit prices can carry more
/// than cents (`0.833` for a weighed banana), so this is wider than 2.
const KEPT_PLACES: u32 = 4;

/// Converts a store's JSON number to a decimal, rounding away float noise.
///
/// `None` for a missing, non-finite or negative value: a store sending one of
/// those has no usable price for the product.
pub fn from_wire(value: Option<f64>) -> Option<Decimal> {
    let value = value?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    Decimal::try_from(value)
        .ok()
        .map(|d| d.round_dp(KEPT_PLACES).normalize())
}

/// Like [`from_wire`], but treats zero as "no value".
///
/// The stores use `0` to mean "no previous price" (`was: 0`) and "no
/// saving", so a zero there is an absence rather than a free product.
pub fn positive_from_wire(value: Option<f64>) -> Option<Decimal> {
    from_wire(value).filter(|d| !d.is_zero())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn keeps_cents_exactly() {
        assert_eq!(from_wire(Some(5.95)), Some(dec!(5.95)));
        assert_eq!(from_wire(Some(0.833)), Some(dec!(0.833)));
    }

    #[test]
    fn rejects_missing_and_nonsense() {
        assert_eq!(from_wire(None), None);
        assert_eq!(from_wire(Some(f64::NAN)), None);
        assert_eq!(from_wire(Some(-1.0)), None);
    }

    #[test]
    fn zero_is_an_absence_where_the_store_means_it() {
        assert_eq!(positive_from_wire(Some(0.0)), None);
        assert_eq!(positive_from_wire(Some(2.3)), Some(dec!(2.3)));
    }
}
