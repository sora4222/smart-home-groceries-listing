//! Reading a running fill program's progress from the store page.
//!
//! The store page has no access to the app's commands. The fill program
//! instead keeps its progress in `window.__groceryFill`
//! (`frontend/src/lib/store-tab/bookmarklet.ts`), and the app reads it by
//! evaluating [`READ_STATUS`]. A page that loads again loses it.

use serde::Deserialize;

/// Evaluates to the program's progress as JSON text, or `"null"`.
pub const READ_STATUS: &str = "JSON.stringify(window.__groceryFill || null)";

/// The program's progress, as last read from the page.
#[derive(Debug, PartialEq, Eq)]
pub enum PageStatus {
    Running,
    Completed(String),
    Failed(String),
    /// No progress on the page: it loaded again, or the program never ran.
    Gone,
}

#[derive(Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
enum Reported {
    Running,
    Completed { message: String },
    Failed { message: String },
}

/// Reads the result of evaluating [`READ_STATUS`]. Webviews hand it back
/// JSON-encoded, so the JSON text arrives once more as a JSON string.
pub fn parse(evaluated: &str) -> PageStatus {
    let text = match serde_json::from_str::<serde_json::Value>(evaluated) {
        Ok(serde_json::Value::String(inner)) => inner,
        _ => evaluated.to_string(),
    };
    match serde_json::from_str::<Option<Reported>>(&text) {
        Ok(Some(Reported::Running)) => PageStatus::Running,
        Ok(Some(Reported::Completed { message })) => PageStatus::Completed(message),
        Ok(Some(Reported::Failed { message })) => PageStatus::Failed(message),
        Ok(None) | Err(_) => PageStatus::Gone,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_state_from_the_encoded_string() {
        assert_eq!(parse(r#""{\"state\":\"running\"}""#), PageStatus::Running);
        assert_eq!(
            parse(r#""{\"state\":\"completed\",\"message\":\"Added 5 products.\"}""#),
            PageStatus::Completed("Added 5 products.".into())
        );
        assert_eq!(
            parse(r#""{\"state\":\"failed\",\"message\":\"Fill trolley failed: 401\"}""#),
            PageStatus::Failed("Fill trolley failed: 401".into())
        );
    }

    #[test]
    fn reads_plain_json_too() {
        assert_eq!(parse(r#"{"state":"running"}"#), PageStatus::Running);
    }

    #[test]
    fn a_reloaded_page_or_anything_unexpected_is_gone() {
        assert_eq!(parse(r#""null""#), PageStatus::Gone);
        assert_eq!(parse("null"), PageStatus::Gone);
        assert_eq!(parse(""), PageStatus::Gone);
        assert_eq!(parse(r#""{\"state\":\"paid\"}""#), PageStatus::Gone);
    }
}
