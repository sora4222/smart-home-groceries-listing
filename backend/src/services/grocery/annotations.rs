//! Tidying the free-text note a household member attaches to a list item.

/// Trims an annotation, treating a blank one as no annotation at all.
pub fn clean_note(note: Option<&str>) -> Option<String> {
    note.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::clean_note;

    #[test]
    fn a_blank_note_is_no_note() {
        assert_eq!(clean_note(Some("   ")), None);
        assert_eq!(clean_note(Some("")), None);
        assert_eq!(clean_note(None), None);
    }

    #[test]
    fn a_note_is_trimmed() {
        assert_eq!(
            clean_note(Some("  the recycled one ")).as_deref(),
            Some("the recycled one")
        );
    }
}
