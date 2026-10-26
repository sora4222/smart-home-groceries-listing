//! How many of each product to buy when an item has several at one store.
//!
//! Pure. The counts at a store always add up to the item's quantity: a lone
//! product buys all of it, and stored counts that no longer add up (the
//! item's quantity was edited) are corrected here when read, never by a
//! write. Products are given oldest choice first.

/// The count each product buys, in the order given, adding up to `total`.
///
/// Stored counts are kept where they fit. A shortfall goes to the oldest
/// product; an excess is taken from the newest first, down to one each. A
/// product can get zero only when there are more products than `total`.
pub fn allocate(stored: &[Option<i32>], total: i32) -> Vec<i32> {
    let total = total.max(0);
    if stored.len() <= 1 {
        return stored.iter().map(|_| total).collect();
    }
    let mut counts: Vec<i32> = stored.iter().map(|q| q.unwrap_or(1).max(1)).collect();
    let sum: i32 = counts.iter().sum();
    if sum < total {
        counts[0] += total - sum;
    } else {
        take_from_newest(&mut counts, sum - total, 1);
        let left: i32 = counts.iter().sum();
        take_from_newest(&mut counts, left - total, 0);
    }
    counts
}

/// The counts with one room made for a new product: one taken from the
/// product buying the most (the oldest on a tie). `None` when every product
/// already buys only one.
pub fn make_room(counts: &[i32]) -> Option<Vec<i32>> {
    let (largest, _) = counts
        .iter()
        .enumerate()
        .rev()
        .max_by_key(|(_, count)| **count)?;
    if counts[largest] < 2 {
        return None;
    }
    let mut next = counts.to_vec();
    next[largest] -= 1;
    Some(next)
}

/// The counts with product `index` buying `wanted`, the others adjusted
/// (newest first, each keeping at least one) so they still add up to the
/// same total. `None` when that cannot be done.
pub fn rebalance(counts: &[i32], index: usize, wanted: i32) -> Option<Vec<i32>> {
    let total: i32 = counts.iter().sum();
    let others = i32::try_from(counts.len()).ok()? - 1;
    if index >= counts.len() || wanted < 1 || wanted > total - others {
        return None;
    }
    let mut next = counts.to_vec();
    let change = wanted - next[index];
    next[index] = wanted;
    if change < 0 {
        // Give the freed count to the newest other product.
        let newest = (0..next.len()).rev().find(|&i| i != index)?;
        next[newest] -= change;
    } else {
        let mut need = change;
        for i in (0..next.len()).rev().filter(|&i| i != index) {
            let spare = (next[i] - 1).min(need);
            next[i] -= spare;
            need -= spare;
        }
    }
    Some(next)
}

/// Takes `amount` off the newest counts first, leaving each at `floor`.
fn take_from_newest(counts: &mut [i32], mut amount: i32, floor: i32) {
    for count in counts.iter_mut().rev() {
        if amount <= 0 {
            break;
        }
        let spare = (*count - floor).min(amount);
        *count -= spare;
        amount -= spare;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lone_product_buys_the_whole_quantity() {
        assert_eq!(allocate(&[None], 4), vec![4]);
        assert_eq!(allocate(&[Some(1)], 4), vec![4]);
        assert!(allocate(&[], 4).is_empty());
    }

    #[test]
    fn stored_counts_that_add_up_are_kept() {
        assert_eq!(allocate(&[Some(3), Some(1)], 4), vec![3, 1]);
    }

    #[test]
    fn a_raised_quantity_goes_to_the_oldest_product() {
        assert_eq!(allocate(&[Some(1), Some(1)], 5), vec![4, 1]);
    }

    #[test]
    fn a_lowered_quantity_comes_off_the_newest_first() {
        assert_eq!(allocate(&[Some(2), Some(3)], 3), vec![2, 1]);
        assert_eq!(allocate(&[Some(3), Some(3)], 2), vec![1, 1]);
    }

    #[test]
    fn more_products_than_the_quantity_leaves_the_newest_at_zero() {
        assert_eq!(allocate(&[Some(1), Some(1), Some(1)], 2), vec![1, 1, 0]);
    }

    #[test]
    fn room_is_made_from_the_largest_count() {
        assert_eq!(make_room(&[1, 3]), Some(vec![1, 2]));
        assert_eq!(make_room(&[2, 2]), Some(vec![1, 2]));
        assert_eq!(make_room(&[1, 1]), None);
    }

    #[test]
    fn raising_one_count_takes_from_the_newest_others() {
        assert_eq!(rebalance(&[1, 2, 3], 0, 4), Some(vec![4, 1, 1]));
    }

    #[test]
    fn lowering_one_count_gives_to_the_newest_other() {
        assert_eq!(rebalance(&[4, 1], 0, 2), Some(vec![2, 3]));
        assert_eq!(rebalance(&[1, 4], 1, 2), Some(vec![3, 2]));
    }

    #[test]
    fn a_count_that_leaves_no_room_for_the_others_is_refused() {
        assert_eq!(rebalance(&[1, 1, 1], 0, 2), None);
        assert_eq!(rebalance(&[2, 1], 0, 0), None);
        assert_eq!(rebalance(&[2, 1], 5, 1), None);
    }
}
