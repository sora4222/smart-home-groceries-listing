//! Tidying the filter terms ("3 ply", "organic") that narrow a product search.
//!
//! Terms reach the backend from the web app's chip inputs and from item rules;
//! both are stored in `TEXT[]` columns bounded by the same `CHECK`, so both go
//! through [`clean`] first. [`merge`] combines an item's own chips with the
//! ones its rules add.

use crate::models::schemas::MAX_FILTER_TERMS;

/// Tidies a term list: trims each term, drops blanks, removes
/// case-insensitive repeats keeping the first spelling, and caps the count at
/// [`MAX_FILTER_TERMS`].
///
/// The bounds are also `CHECK` constraints on the columns; doing the work here
/// means a client that sends `["3 Ply", " 3 ply "]` gets one chip instead of a
/// 422.
pub fn clean(terms: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut cleaned: Vec<String> = Vec::new();
    for term in terms {
        let term = term.trim();
        if term.is_empty() || cleaned.len() >= MAX_FILTER_TERMS {
            continue;
        }
        let key = term.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        cleaned.push(term.to_string());
    }
    cleaned
}

/// Appends `extra` to `first` and tidies the result, so `first`'s terms keep
/// their place and spelling and `extra` fills whatever room is left.
pub fn merge(first: &[String], extra: &[String]) -> Vec<String> {
    let combined: Vec<String> = first.iter().chain(extra).cloned().collect();
    clean(&combined)
}

#[cfg(test)]
mod tests {
    use super::{clean, merge};

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn merge_keeps_the_first_lists_spelling_and_order() {
        assert_eq!(
            merge(
                &strings(&["Recycled", "bulk"]),
                &strings(&["3 ply", "recycled"])
            ),
            vec!["Recycled", "bulk", "3 ply"]
        );
    }

    #[test]
    fn merge_fills_only_the_room_left() {
        let first: Vec<String> = (0..9).map(|n| format!("own {n}")).collect();
        let merged = merge(&first, &strings(&["a", "b"]));
        assert_eq!(merged.len(), 10);
        assert_eq!(merged[9], "a");
    }

    #[test]
    fn terms_are_trimmed_and_blanks_dropped() {
        let terms = vec![
            "  3 ply ".to_string(),
            "  ".to_string(),
            "recycled".to_string(),
        ];
        assert_eq!(clean(&terms), vec!["3 ply", "recycled"]);
    }

    #[test]
    fn a_repeat_keeps_its_first_spelling() {
        let terms = vec![
            "3 Ply".to_string(),
            "3 ply".to_string(),
            " 3 PLY ".to_string(),
        ];
        assert_eq!(clean(&terms), vec!["3 Ply"]);
    }

    #[test]
    fn terms_are_capped_at_the_column_limit() {
        let terms: Vec<String> = (0..25).map(|n| format!("term {n}")).collect();
        assert_eq!(clean(&terms).len(), 10);
    }

    #[test]
    fn an_empty_list_stays_empty() {
        assert!(clean(&[]).is_empty());
    }
}
