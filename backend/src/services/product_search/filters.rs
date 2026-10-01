//! Applying an item's filter chips to a store's results.
//!
//! A product passes when every chip appears in its brand, name or pack size.
//! Matching ignores case and punctuation, and treats "3-ply", "3 ply" and
//! "3ply" as the same chip — stores are not consistent about which they
//! write.

use crate::services::stores::Product;

/// Keeps the products that match every term.
pub fn apply(products: Vec<Product>, terms: &[String]) -> Vec<Product> {
    let terms: Vec<Normalised> = terms
        .iter()
        .map(|t| Normalised::new(t))
        .filter(|t| !t.squashed.is_empty())
        .collect();
    if terms.is_empty() {
        return products;
    }
    products
        .into_iter()
        .filter(|product| {
            let text = Normalised::new(&product.searchable_text());
            terms.iter().all(|term| text.contains(term))
        })
        .collect()
}

/// Text reduced to lowercase words.
struct Normalised {
    words: Vec<String>,
    /// The words with no spaces at all: `3ply`.
    squashed: String,
}

impl Normalised {
    fn new(text: &str) -> Self {
        let words: Vec<String> = text
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect();
        let squashed = words.concat();
        Self { words, squashed }
    }

    /// `term` is a run of whole consecutive words here, with spacing
    /// ignored: "3 ply" matches "3ply" and "3-ply", but "apple" does not
    /// match "pineapple".
    fn contains(&self, term: &Normalised) -> bool {
        (0..self.words.len()).any(|start| {
            let mut run = String::new();
            self.words[start..].iter().any(|word| {
                run.push_str(word);
                run == term.squashed
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::product::fixtures::product;
    use crate::services::stores::Store;
    use rust_decimal_macros::dec;

    fn names(products: &[Product]) -> Vec<&str> {
        products.iter().map(|p| p.name.as_str()).collect()
    }

    fn catalogue() -> Vec<Product> {
        vec![
            product(Store::Coles, "3 Ply Toilet Rolls", dec!(13)),
            product(Store::Coles, "Toilet Paper 2-Ply", dec!(9)),
            product(Store::Coles, "Kleenex 3-ply Toilet Paper", dec!(12)),
        ]
    }

    #[test]
    fn no_chips_keeps_everything() {
        assert_eq!(apply(catalogue(), &[]).len(), 3);
    }

    #[test]
    fn a_chip_matches_regardless_of_hyphens_and_case() {
        let kept = apply(catalogue(), &["3 PLY".to_string()]);
        assert_eq!(
            names(&kept),
            ["3 Ply Toilet Rolls", "Kleenex 3-ply Toilet Paper"]
        );
    }

    #[test]
    fn a_chip_written_without_a_space_still_matches() {
        let kept = apply(catalogue(), &["3ply".to_string()]);
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn every_chip_must_match() {
        let kept = apply(catalogue(), &["3 ply".to_string(), "kleenex".to_string()]);
        assert_eq!(names(&kept), ["Kleenex 3-ply Toilet Paper"]);
    }

    #[test]
    fn a_chip_does_not_match_inside_another_word() {
        let kept = apply(
            vec![product(Store::Coles, "Pineapple Pieces", dec!(3))],
            &["apple".to_string()],
        );
        assert!(kept.is_empty(), "{:?}", names(&kept));
    }
}
