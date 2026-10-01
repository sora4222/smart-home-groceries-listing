//! Finding the Next.js build id in a Coles page.
//!
//! Coles' data routes are addressed by the id of the website build that
//! serves them (`/_next/data/<buildId>/...`). Every page embeds it in its
//! `__NEXT_DATA__` script as `"buildId":"..."`; it changes whenever Coles
//! deploys.

/// The marker before the id in the page's embedded JSON.
const MARKER: &str = "\"buildId\":\"";

/// Extracts the build id from a page's HTML, or `None` if it has none (for
/// example a bot-protection challenge page instead of the real site).
pub fn extract(html: &str) -> Option<String> {
    let start = html.find(MARKER)? + MARKER.len();
    let rest = &html[start..];
    let id = &rest[..rest.find('"')?];
    is_plausible(id).then(|| id.to_string())
}

/// Build ids are short and URL-safe; anything else is not one, and must not
/// be pasted into a URL path.
fn is_plausible(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_id_in_next_data() {
        let html = include_str!("../../../../tests/fixtures/coles/home.html");
        assert_eq!(
            extract(html).as_deref(),
            Some("20260929.4-53e2baf1c46ef35378fc9f730ad7af8617974443")
        );
    }

    #[test]
    fn a_page_without_one_has_none() {
        assert_eq!(extract("<html>Access denied</html>"), None);
    }

    #[test]
    fn refuses_an_id_that_could_change_the_url_path() {
        assert_eq!(extract(r#"{"buildId":"../../admin"}"#), None);
    }
}
