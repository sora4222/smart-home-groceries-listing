//! Sharing an order's delivery fee across its lines.
//!
//! Each line gets a share in proportion to what it cost, in whole cents. The
//! last line takes whatever rounding left over, so the shares always add up
//! to the fee exactly. Pure: no I/O.

use rust_decimal::{Decimal, RoundingStrategy};

/// Each line's share of `fee`, in the order of `line_totals`.
///
/// When every line cost nothing, the fee is shared evenly instead.
pub fn split(fee: Decimal, line_totals: &[Decimal]) -> Vec<Decimal> {
    let Some(last) = line_totals.len().checked_sub(1) else {
        return Vec::new();
    };
    let sum: Decimal = line_totals.iter().sum();
    let count = Decimal::from(line_totals.len());
    let mut shares: Vec<Decimal> = line_totals
        .iter()
        .map(|total| {
            let share = if sum.is_zero() {
                fee / count
            } else {
                fee * total / sum
            };
            share.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        })
        .collect();
    let given: Decimal = shares[..last].iter().sum();
    shares[last] = (fee - given).max(Decimal::ZERO);
    shares
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn shares_follow_what_each_line_cost() {
        assert_eq!(
            split(dec!(10), &[dec!(30), dec!(10)]),
            vec![dec!(7.50), dec!(2.50)]
        );
    }

    #[test]
    fn the_shares_add_up_to_the_fee_exactly() {
        let shares = split(dec!(10), &[dec!(1), dec!(1), dec!(1)]);
        assert_eq!(shares, vec![dec!(3.33), dec!(3.33), dec!(3.34)]);
        assert_eq!(shares.iter().sum::<Decimal>(), dec!(10));
    }

    #[test]
    fn lines_that_cost_nothing_share_evenly() {
        assert_eq!(split(dec!(4), &[dec!(0), dec!(0)]), vec![dec!(2), dec!(2)]);
    }

    #[test]
    fn no_fee_gives_zero_shares() {
        assert_eq!(
            split(Decimal::ZERO, &[dec!(5), dec!(6)]),
            vec![dec!(0), dec!(0)]
        );
    }

    #[test]
    fn no_lines_gives_no_shares() {
        assert!(split(dec!(9), &[]).is_empty());
    }
}
