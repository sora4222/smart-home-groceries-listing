//! One request, as the access log records it.

/// The user id recorded when no route verified a session: intake endpoints,
/// the health check, a missing or bad token.
pub const UNAUTHENTICATED: &str = "unauthenticated";

/// Everything the access log keeps about one answered request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessLogEntry {
    /// The address that opened the connection, when the server knows it.
    pub source_ip: Option<String>,
    /// What a forwarding header claimed the client was. Shown, never trusted.
    pub forwarded_for: Option<String>,
    /// The signed-in household member, or [`UNAUTHENTICATED`].
    pub user_id: String,
    pub method: String,
    /// The path only — never the query string.
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u32,
}

/// The user id to record: the signed-in user, or [`UNAUTHENTICATED`].
pub fn user_label(signed_in: Option<&str>) -> String {
    signed_in.unwrap_or(UNAUTHENTICATED).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signed_in_user_is_recorded_by_id() {
        assert_eq!(user_label(Some("user_abc")), "user_abc");
    }

    #[test]
    fn no_user_is_recorded_as_unauthenticated() {
        assert_eq!(user_label(None), "unauthenticated");
    }
}
