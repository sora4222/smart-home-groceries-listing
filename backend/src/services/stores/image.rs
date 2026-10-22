//! Product picture links, kept only when they are safe to put in a page.

/// `url` when it is an `https://` link, else `None`.
///
/// The web app puts the link straight into an `<img src>`, so anything else
/// a store might send (plain `http`, `javascript:`, an empty string) is
/// dropped and the product shows without a picture.
pub fn https_only(url: Option<&str>) -> Option<String> {
    url.map(str::trim)
        .filter(|url| url.starts_with("https://") && url.len() > "https://".len())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_an_https_link() {
        assert_eq!(
            https_only(Some(" https://cdn.example/a.jpg ")).as_deref(),
            Some("https://cdn.example/a.jpg")
        );
    }

    #[test]
    fn drops_anything_else() {
        for url in [
            "",
            "https://",
            "http://cdn.example/a.jpg",
            "javascript:alert(1)",
        ] {
            assert_eq!(https_only(Some(url)), None, "{url}");
        }
        assert_eq!(https_only(None), None);
    }
}
