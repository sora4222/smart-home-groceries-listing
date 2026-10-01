//! The text sent to a store's search box for a list item.

/// The item's name followed by its filter chips, e.g. "toilet paper 3 ply".
///
/// Chips are sent to the store as well as applied afterwards
/// ([`super::filters`]): the store's own search then ranks matching products
/// first, so the narrowed list is not just the leftovers of a broad one.
/// Whitespace is collapsed so the same item always makes the same query
/// (and so hits the search cache).
pub fn for_item(name: &str, filter_terms: &[String]) -> String {
    std::iter::once(name)
        .chain(filter_terms.iter().map(String::as_str))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_the_name_and_the_chips() {
        assert_eq!(
            for_item("toilet  paper", &["3 ply".to_string()]),
            "toilet paper 3 ply"
        );
    }

    #[test]
    fn an_item_without_chips_is_its_name() {
        assert_eq!(for_item(" milk ", &[]), "milk");
    }
}
