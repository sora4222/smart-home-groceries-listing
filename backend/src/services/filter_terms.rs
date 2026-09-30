//! Tidying the filter terms ("3 ply", "organic") that narrow a product search.
//!
//! Terms reach the backend from the web app's chip inputs and from item rules;
//! both are stored in `TEXT[]` columns bounded by the same `CHECK`, so both go
//! through [`clean`] first.

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

#[cfg(test)]
mod tests {
    use super::clean;

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
