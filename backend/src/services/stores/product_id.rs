//! What a store product id looks like.
//!
//! Both stores number their products: a Woolworths stockcode (`722`) and a
//! Coles product id (`8150288`) are plain digits. An id goes into a store
//! URL path, so anything else is refused before a request is made — it
//! could only be a mistake or an attempt to change the path.

/// Longest id accepted. Real ids are 3 to 8 digits.
const MAX_DIGITS: usize = 12;

/// `id` is a store product id: 1 to 12 ASCII digits.
pub fn is_valid(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_DIGITS && id.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_digits_are_ids() {
        assert!(is_valid("722"));
        assert!(is_valid("8150288"));
    }

    #[test]
    fn anything_else_is_refused() {
        assert!(!is_valid(""));
        assert!(!is_valid("w-milk-2l"));
        assert!(!is_valid("../admin"));
        assert!(!is_valid("1234567890123"));
        assert!(!is_valid(" 722"));
    }
}
