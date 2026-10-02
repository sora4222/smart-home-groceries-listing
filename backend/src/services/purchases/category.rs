//! Putting a bought product in a spending category.
//!
//! Woolworths and Coles name their categories differently ("Milk",
//! "Long-Life Milk", "Fruit & Vegetables"), so the analysis maps every product
//! into one short, shared list. The store's own category is tried first; when
//! the store gave none, or none of its words match, the product's name is
//! tried. Nothing matched → "Other".
//!
//! Matching is by whole words, so "cat" never matches "category". When several
//! keywords match, the longest wins ("peanut butter" beats "butter"); on a tie
//! the category listed first wins. Pure: no I/O.

use crate::models::db::CategorySource;

/// The category for anything no keyword matched.
pub const OTHER: &str = "Other";

mod keywords;

use keywords::CATEGORIES;

/// A product's category and where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Categorised {
    pub category: &'static str,
    pub source: CategorySource,
}

/// The category for a product, from the store's category, else its name.
pub fn categorise(store_category: Option<&str>, product_name: &str) -> Categorised {
    if let Some(category) = store_category.and_then(best_match) {
        return Categorised {
            category,
            source: CategorySource::Store,
        };
    }
    match best_match(product_name) {
        Some(category) => Categorised {
            category,
            source: CategorySource::Name,
        },
        None => Categorised {
            category: OTHER,
            source: CategorySource::None,
        },
    }
}

/// The category whose longest matching keyword is longest, if any matches.
fn best_match(text: &str) -> Option<&'static str> {
    let words = words(text);
    let mut best: Option<(&'static str, usize)> = None;
    for (category, keywords) in CATEGORIES {
        for keyword in *keywords {
            let keyword_words: Vec<&str> = keyword.split(' ').collect();
            let longer = best.is_none_or(|(_, len)| keyword_words.len() > len);
            if longer && contains_phrase(&words, &keyword_words) {
                best = Some((category, keyword_words.len()));
            }
        }
    }
    best.map(|(category, _)| category)
}

/// The text's words, lowercased, split on anything that is not a letter or digit.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

/// `phrase` appears in `words` as consecutive words, allowing plurals.
fn contains_phrase(words: &[String], phrase: &[&str]) -> bool {
    words
        .windows(phrase.len())
        .any(|window| window.iter().zip(phrase).all(|(w, p)| same_word(w, p)))
}

/// `word` is `keyword`, or its plural with -s or -es.
fn same_word(word: &str, keyword: &str) -> bool {
    word.strip_prefix(keyword)
        .is_some_and(|rest| matches!(rest, "" | "s" | "es"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category(store: Option<&str>, name: &str) -> &'static str {
        categorise(store, name).category
    }

    #[test]
    fn the_stores_category_comes_first() {
        let found = categorise(Some("Long-Life Milk"), "Barista Oat Something");
        assert_eq!(found.category, "Dairy & eggs");
        assert_eq!(found.source, CategorySource::Store);
    }

    #[test]
    fn the_name_is_used_when_the_store_gave_no_category() {
        let found = categorise(None, "Free Range Eggs 12 pack");
        assert_eq!(found.category, "Dairy & eggs");
        assert_eq!(found.source, CategorySource::Name);
    }

    #[test]
    fn the_name_is_used_when_the_stores_category_matches_nothing() {
        let found = categorise(Some("Specials"), "Bananas");
        assert_eq!(found.category, "Fruit & veg");
        assert_eq!(found.source, CategorySource::Name);
    }

    #[test]
    fn nothing_matched_is_other() {
        let found = categorise(None, "Gift card");
        assert_eq!(found.category, OTHER);
        assert_eq!(found.source, CategorySource::None);
    }

    #[test]
    fn the_longest_keyword_wins() {
        assert_eq!(category(None, "Smooth Peanut Butter"), "Pantry");
        assert_eq!(category(None, "Vanilla Ice Cream"), "Frozen");
        assert_eq!(category(None, "Milk Chocolate Block"), "Snacks & sweets");
        assert_eq!(category(Some("Toilet Paper"), "Kleenex 3 Ply"), "Household");
    }

    #[test]
    fn on_a_tie_the_category_listed_first_wins() {
        // "milk" (Dairy) and "oat" (Pantry) are one word each.
        assert_eq!(category(None, "Barista Oat Milk"), "Dairy & eggs");
    }

    #[test]
    fn plurals_match() {
        assert_eq!(category(None, "Truss Tomatoes"), "Fruit & veg");
        assert_eq!(category(Some("Soft Drinks"), "Pepsi Max"), "Drinks");
    }

    #[test]
    fn only_whole_words_match() {
        // "cat" is inside "Category" and "delicious", "deli" inside "delicious".
        assert_eq!(category(Some("Category X"), "Delicious thing"), OTHER);
    }

    #[test]
    fn case_and_punctuation_do_not_matter() {
        assert_eq!(category(Some("FRUIT & VEGETABLES"), "x"), "Fruit & veg");
    }
}
