//! Field limits and validators shared by more than one request body.
//!
//! Each limit mirrors a `CHECK` constraint in `migrations/`, so a body the
//! validator lets through is a row the database will accept.

use validator::ValidationError;

/// Upper bound on a single item's quantity, matching the web app's input.
pub const MAX_QUANTITY: i32 = 999;
/// Upper bound on an item name, matching the `name` column's CHECK constraint.
pub const MAX_NAME_LEN: u64 = 200;
/// Upper bound on an item's free-text note, matching the `note` CHECK.
pub const MAX_NOTE_LEN: u64 = 500;
/// How many filter chips one item may carry, matching the `filter_terms` CHECK.
pub const MAX_FILTER_TERMS: usize = 10;
/// Longest single filter term, matching the `filter_terms` CHECK.
pub const MAX_FILTER_TERM_LEN: usize = 60;

/// The quantity a body gets when it does not say.
pub(crate) fn default_quantity() -> i32 {
    1
}

/// Rejects a chip list the `filter_terms` CHECK constraint would also reject.
///
/// The service trims and de-duplicates terms before storing them, so this only
/// has to catch what trimming cannot fix: too many chips, or one too long.
pub(crate) fn validate_filter_terms(terms: &[String]) -> Result<(), ValidationError> {
    if terms.len() > MAX_FILTER_TERMS {
        return Err(ValidationError::new("too_many_filter_terms"));
    }
    if terms
        .iter()
        .any(|term| term.trim().chars().count() > MAX_FILTER_TERM_LEN)
    {
        return Err(ValidationError::new("filter_term_too_long"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_filter_terms;

    #[test]
    fn ten_short_terms_are_accepted() {
        let terms: Vec<String> = (0..10).map(|n| format!("term {n}")).collect();
        assert!(validate_filter_terms(&terms).is_ok());
    }

    #[test]
    fn eleven_terms_are_too_many() {
        let terms: Vec<String> = (0..11).map(|n| format!("term {n}")).collect();
        assert!(validate_filter_terms(&terms).is_err());
    }

    #[test]
    fn a_term_over_sixty_characters_is_too_long() {
        assert!(validate_filter_terms(&["x".repeat(61)]).is_err());
    }

    #[test]
    fn surrounding_whitespace_does_not_count_towards_the_length() {
        assert!(validate_filter_terms(&[format!("  {}  ", "x".repeat(60))]).is_ok());
    }
}
