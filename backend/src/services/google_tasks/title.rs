//! Reading a task title as an item and a quantity (pure).
//!
//! People type "2 oat milk", "oat milk x2", "2x oat milk" or just "oat milk".
//! Anything else stays part of the name; the household corrects names in
//! Pending Requests.

use crate::models::schemas::{MAX_NAME_LEN, MAX_QUANTITY};

/// Longest raw text kept, matching the `raw_text` CHECK constraint.
const MAX_RAW_LEN: usize = 2000;

/// One task title, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTitle {
    /// The title as typed, trimmed and cut to fit the column.
    pub raw_text: String,
    /// The item name.
    pub name: String,
    /// How many, 1 to [`MAX_QUANTITY`].
    pub quantity: i32,
}

/// Reads a title. `None` for a blank title, which is not an item.
pub fn parse_title(title: &str) -> Option<ParsedTitle> {
    let raw = collapse(title);
    if raw.is_empty() {
        return None;
    }
    let (name, quantity) = split_quantity(&raw);
    let name = cut(&name, MAX_NAME_LEN as usize);
    if name.is_empty() {
        return None;
    }
    Some(ParsedTitle {
        raw_text: cut(&raw, MAX_RAW_LEN),
        name,
        quantity: quantity.clamp(1, MAX_QUANTITY),
    })
}

/// Splits a leading or trailing count off the name. The count is kept only
/// when a name is left over, so "7up" and "2" stay names.
fn split_quantity(text: &str) -> (String, i32) {
    let words: Vec<&str> = text.split(' ').collect();
    if words.len() > 1 {
        if let Some(count) = count_word(words[0]) {
            let rest = &words[1..];
            // "2 x milk" — a lone "x" after the number belongs to the count.
            let rest = match rest {
                [x, tail @ ..] if is_times(x) && !tail.is_empty() => tail,
                _ => rest,
            };
            return (rest.join(" "), count);
        }
        let last = words[words.len() - 1];
        if let Some(count) = trailing_count(last) {
            return (words[..words.len() - 1].join(" "), count);
        }
        if words.len() > 2 && is_times(words[words.len() - 2]) {
            if let Ok(count) = last.parse::<i32>() {
                return (words[..words.len() - 2].join(" "), count);
            }
        }
    }
    (text.to_string(), 1)
}

/// "2" or "2x" at the start.
fn count_word(word: &str) -> Option<i32> {
    let digits = word.strip_suffix(['x', 'X', '×']).unwrap_or(word);
    digits.parse::<i32>().ok()
}

/// "x2" or "×2" at the end.
fn trailing_count(word: &str) -> Option<i32> {
    word.strip_prefix(['x', 'X', '×'])
        .and_then(|digits| digits.parse::<i32>().ok())
}

fn is_times(word: &str) -> bool {
    matches!(word, "x" | "X" | "×")
}

/// Trims and squeezes runs of whitespace (including new lines) to one space.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Keeps at most `max` characters.
fn cut(text: &str, max: usize) -> String {
    text.chars()
        .take(max)
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(title: &str) -> (String, i32) {
        let parsed = parse_title(title).expect("a title");
        (parsed.name, parsed.quantity)
    }

    #[test]
    fn a_plain_name_is_one() {
        assert_eq!(read("oat milk"), ("oat milk".into(), 1));
    }

    #[test]
    fn a_count_at_the_start_or_end_is_the_quantity() {
        assert_eq!(read("2 oat milk"), ("oat milk".into(), 2));
        assert_eq!(read("2x oat milk"), ("oat milk".into(), 2));
        assert_eq!(read("2 x oat milk"), ("oat milk".into(), 2));
        assert_eq!(read("oat milk x2"), ("oat milk".into(), 2));
        assert_eq!(read("oat milk ×3"), ("oat milk".into(), 3));
        assert_eq!(read("oat milk x 4"), ("oat milk".into(), 4));
    }

    #[test]
    fn a_number_that_is_the_whole_name_stays_the_name() {
        assert_eq!(read("7up"), ("7up".into(), 1));
        assert_eq!(read("2"), ("2".into(), 1));
        assert_eq!(read("x2"), ("x2".into(), 1));
    }

    #[test]
    fn the_quantity_is_kept_in_range() {
        assert_eq!(read("0 eggs"), ("eggs".into(), 1));
        assert_eq!(read("5000 eggs"), ("eggs".into(), MAX_QUANTITY));
    }

    #[test]
    fn whitespace_is_tidied_and_blank_titles_are_not_items() {
        assert_eq!(read("  oat \n milk "), ("oat milk".into(), 1));
        assert!(parse_title("   ").is_none());
        assert!(parse_title("").is_none());
    }

    #[test]
    fn long_titles_are_cut_to_fit() {
        let parsed = parse_title(&"a".repeat(3000)).unwrap();
        assert_eq!(parsed.name.chars().count(), MAX_NAME_LEN as usize);
        assert_eq!(parsed.raw_text.chars().count(), MAX_RAW_LEN);
    }

    #[test]
    fn the_raw_text_is_what_was_typed() {
        assert_eq!(parse_title("2 oat milk").unwrap().raw_text, "2 oat milk");
    }
}
