//! Which list item a spoken name means. Pure, so it is unit-tested alone.
//!
//! Matching is deliberately narrow: the same name ignoring case and spacing,
//! or its plain plural or singular ("eggs" for "egg"). Anything looser would
//! be guessing, and a wrong guess silently takes the wrong thing off the
//! list. When nothing matches, Alexa says so and nothing changes.

use crate::services::grocery::repository::normalise;

/// The normalised names a spoken item may match, the exact form first.
///
/// Empty when the spoken name is only whitespace.
pub fn name_variants(spoken: &str) -> Vec<String> {
    let exact = normalise(spoken);
    if exact.is_empty() {
        return Vec::new();
    }
    let mut variants = vec![exact.clone()];
    let mut push = |candidate: String| {
        if !candidate.is_empty() && !variants.contains(&candidate) {
            variants.push(candidate);
        }
    };
    if let Some(stem) = exact.strip_suffix("es") {
        push(stem.to_string());
    }
    if let Some(stem) = exact.strip_suffix('s') {
        push(stem.to_string());
    } else {
        push(format!("{exact}s"));
        if ["o", "ch", "sh", "x"]
            .iter()
            .any(|end| exact.ends_with(end))
        {
            push(format!("{exact}es"));
        }
    }
    variants
}

#[cfg(test)]
mod tests {
    use super::name_variants;

    #[test]
    fn the_exact_name_comes_first_normalised() {
        assert_eq!(name_variants("  Oat   Milk ")[0], "oat milk");
    }

    #[test]
    fn a_plural_also_matches_the_singular() {
        assert_eq!(name_variants("eggs"), vec!["eggs", "egg"]);
    }

    #[test]
    fn a_singular_also_matches_the_plural() {
        assert_eq!(name_variants("egg"), vec!["egg", "eggs"]);
    }

    #[test]
    fn an_es_plural_matches_its_singular() {
        assert!(name_variants("tomatoes").contains(&"tomato".to_string()));
        assert!(name_variants("tomato").contains(&"tomatoes".to_string()));
    }

    #[test]
    fn only_the_last_letters_change() {
        // "oat milks" is not a reason to match "oats milk".
        assert_eq!(name_variants("oat milk"), vec!["oat milk", "oat milks"]);
    }

    #[test]
    fn whitespace_alone_matches_nothing() {
        assert!(name_variants("   ").is_empty());
    }
}
