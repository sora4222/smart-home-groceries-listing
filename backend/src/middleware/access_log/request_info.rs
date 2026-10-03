//! Reading the access log's facts out of a request: where it came from and
//! what it asked for. Pure functions, so every edge case is unit-tested.

use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::http::{Extensions, HeaderMap};

/// The header Cloudflare sets to the visitor's address.
pub const CF_CONNECTING_IP: &str = "cf-connecting-ip";
/// The common proxy header, used when Cloudflare's is absent.
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";

/// Longest forwarded-for value kept, matching the `access_logs` CHECK.
const MAX_FORWARDED_LEN: usize = 256;
/// Longest path kept, matching the `access_logs` CHECK.
const MAX_PATH_LEN: usize = 2048;

/// The address that opened the connection, when the server was started with
/// connection info (`main.rs` does; the in-process tests do not).
pub fn source_ip(extensions: &Extensions) -> Option<String> {
    extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip().to_string())
}

/// What the forwarding headers claim the client was: `CF-Connecting-IP`
/// first, then `X-Forwarded-For`. Anyone can send either header, so the
/// result is for showing, never for deciding anything.
pub fn forwarded_for(headers: &HeaderMap) -> Option<String> {
    [CF_CONNECTING_IP, X_FORWARDED_FOR]
        .iter()
        .find_map(|name| {
            let value = headers.get(*name)?.to_str().ok()?.trim();
            (!value.is_empty()).then_some(value)
        })
        .map(|value| truncate(value, MAX_FORWARDED_LEN))
}

/// The request path, cut to what the table holds. The query string is never
/// part of it.
pub fn loggable_path(path: &str) -> String {
    truncate(path, MAX_PATH_LEN)
}

/// The first `max` characters of `value`.
fn truncate(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        map
    }

    #[test]
    fn source_ip_reads_the_connection_address() {
        let mut extensions = Extensions::new();
        extensions.insert(ConnectInfo(
            "192.168.1.20:51515".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(source_ip(&extensions).as_deref(), Some("192.168.1.20"));
    }

    #[test]
    fn source_ip_handles_ipv6() {
        let mut extensions = Extensions::new();
        extensions.insert(ConnectInfo("[::1]:8000".parse::<SocketAddr>().unwrap()));
        assert_eq!(source_ip(&extensions).as_deref(), Some("::1"));
    }

    #[test]
    fn source_ip_is_none_without_connection_info() {
        assert_eq!(source_ip(&Extensions::new()), None);
    }

    #[test]
    fn cloudflare_header_wins_over_x_forwarded_for() {
        let map = headers(&[
            (X_FORWARDED_FOR, "10.0.0.1"),
            (CF_CONNECTING_IP, "203.0.113.9"),
        ]);
        assert_eq!(forwarded_for(&map).as_deref(), Some("203.0.113.9"));
    }

    #[test]
    fn x_forwarded_for_is_used_when_cloudflare_is_absent() {
        let map = headers(&[(X_FORWARDED_FOR, " 198.51.100.4, 10.0.0.1 ")]);
        assert_eq!(
            forwarded_for(&map).as_deref(),
            Some("198.51.100.4, 10.0.0.1")
        );
    }

    #[test]
    fn blank_forwarding_headers_count_as_absent() {
        let map = headers(&[(CF_CONNECTING_IP, "  "), (X_FORWARDED_FOR, "198.51.100.4")]);
        assert_eq!(forwarded_for(&map).as_deref(), Some("198.51.100.4"));
        assert_eq!(forwarded_for(&HeaderMap::new()), None);
    }

    #[test]
    fn an_oversized_forwarded_value_is_cut() {
        let long = "1".repeat(1000);
        let map = headers(&[(CF_CONNECTING_IP, &long)]);
        assert_eq!(forwarded_for(&map).unwrap().len(), MAX_FORWARDED_LEN);
    }

    #[test]
    fn an_oversized_path_is_cut() {
        let long = format!("/{}", "a".repeat(5000));
        assert_eq!(loggable_path(&long).len(), MAX_PATH_LEN);
        assert_eq!(loggable_path("/api/health"), "/api/health");
    }
}
