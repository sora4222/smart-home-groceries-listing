//! The HTTP client the stores are reached with, and how its failures map to
//! [`StoreError`].
//!
//! Both stores sit behind bot protection that fingerprints the TLS handshake
//! and HTTP/2 settings. A plain `reqwest` client is refused on sight, so
//! every store request goes through a `wreq` client emulating a real Chrome.
//! Never use `reqwest` against a store URL.

use std::time::Duration;

use serde::de::DeserializeOwned;
use wreq::header::CONTENT_TYPE;
use wreq::Response;
use wreq_util::Emulation;

use super::error::StoreError;

/// The browser the client presents as. Keep it a recent Chrome: an old
/// version is itself a bot signal.
const EMULATED_BROWSER: Emulation = Emulation::Chrome137;

/// Builds a client that looks like Chrome, keeps cookies between requests
/// (the bot-protection cookies a first page visit sets), and gives up after
/// `timeout` so one slow store cannot hold the search up.
pub fn build_client(timeout: Duration) -> Result<wreq::Client, StoreError> {
    wreq::Client::builder()
        .emulation(EMULATED_BROWSER)
        .cookie_store(true)
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
        .map_err(|err| StoreError::Unreachable(format!("building the HTTP client: {err}")))
}

/// Maps a transport failure (DNS, connect, timeout, reset) to a store error.
pub fn transport_error(err: wreq::Error) -> StoreError {
    if err.is_timeout() {
        StoreError::Unreachable("timed out".into())
    } else {
        StoreError::Unreachable(err.to_string())
    }
}

/// The error a non-success status means, or `None` for a 2xx.
pub fn status_error(status: u16) -> Option<StoreError> {
    match status {
        200..=299 => None,
        401 | 403 | 429 => Some(StoreError::Blocked { status }),
        500..=599 => Some(StoreError::Unreachable(format!("HTTP {status}"))),
        _ => Some(StoreError::UnexpectedResponse(format!("HTTP {status}"))),
    }
}

/// Checks a response's status, then reads it as JSON of type `T`.
///
/// An HTML page where JSON was expected is a bot-protection challenge, so it
/// is reported as [`StoreError::Blocked`] rather than as a parse failure.
pub async fn read_json<T: DeserializeOwned>(response: Response) -> Result<T, StoreError> {
    let status = response.status().as_u16();
    if let Some(err) = status_error(status) {
        return Err(err);
    }
    if is_html(&response) {
        return Err(StoreError::Blocked { status });
    }
    let bytes = response.bytes().await.map_err(transport_error)?;
    if looks_like_markup(&bytes) {
        return Err(StoreError::Blocked { status });
    }
    serde_json::from_slice(&bytes).map_err(|err| StoreError::UnexpectedResponse(err.to_string()))
}

/// Checks a response's status, then reads it as text.
pub async fn read_text(response: Response) -> Result<String, StoreError> {
    let status = response.status().as_u16();
    if let Some(err) = status_error(status) {
        return Err(err);
    }
    response.text().await.map_err(transport_error)
}

/// The body is a page rather than JSON, whatever its headers claim.
fn looks_like_markup(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .find(|b| !b.is_ascii_whitespace())
        .is_some_and(|first| *first == b'<')
}

/// The response declares itself an HTML page.
fn is_html(response: &Response) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().starts_with("text/html"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bot_protection_statuses_are_blocks() {
        assert_eq!(status_error(403), Some(StoreError::Blocked { status: 403 }));
        assert_eq!(status_error(429), Some(StoreError::Blocked { status: 429 }));
    }

    #[test]
    fn markup_is_recognised_whatever_the_headers_say() {
        assert!(looks_like_markup(b"  <html>"));
        assert!(!looks_like_markup(b"{\"a\":1}"));
        assert!(!looks_like_markup(b""));
    }

    #[test]
    fn server_errors_are_unreachable_and_success_is_fine() {
        assert!(matches!(
            status_error(503),
            Some(StoreError::Unreachable(_))
        ));
        assert_eq!(status_error(200), None);
        assert!(matches!(
            status_error(404),
            Some(StoreError::UnexpectedResponse(_))
        ));
    }
}
