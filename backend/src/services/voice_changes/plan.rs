//! The arithmetic of a spoken remove or reduce. Pure, so the edge cases are
//! unit-tested without a database.

/// The quantity an item ends up with.
///
/// `reduce_by` of `None` ("remove milk") takes the whole item off: `0`.
/// `Some(n)` ("remove two milk") lowers the quantity by `n`, and never below
/// `0` — asking to take off more than is on the list takes the item off.
pub fn quantity_after(current: i32, reduce_by: Option<i32>) -> i32 {
    match reduce_by {
        None => 0,
        Some(amount) => current.saturating_sub(amount).max(0),
    }
}

#[cfg(test)]
mod tests {
    use super::quantity_after;

    #[test]
    fn no_amount_removes_the_whole_item() {
        assert_eq!(quantity_after(3, None), 0);
    }

    #[test]
    fn an_amount_lowers_the_quantity() {
        assert_eq!(quantity_after(3, Some(1)), 2);
    }

    #[test]
    fn reducing_by_the_whole_quantity_removes_the_item() {
        assert_eq!(quantity_after(2, Some(2)), 0);
    }

    #[test]
    fn reducing_by_more_than_is_listed_removes_the_item() {
        assert_eq!(quantity_after(2, Some(999)), 0);
    }
}
