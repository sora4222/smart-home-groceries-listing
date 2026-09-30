//! Shared-secret authentication for the intake endpoints.
//!
//! The generic webhook and the Alexa bridge cannot carry a session JWT, so
//! each is authenticated with a secret shared out of band. Comparison is
//! constant-time: a byte-by-byte `==` leaks the length of the matching prefix
//! through timing, which is enough to recover a secret over many attempts.

use subtle::ConstantTimeEq;

use crate::error::ApiError;

/// Compares a presented secret against the configured one.
///
/// An unset `expected` is a misconfiguration, not an open door: it fails the
/// request with a 500 so the endpoint cannot be left unauthenticated by a
/// missing environment variable.
pub fn verify_shared_secret(
    expected: &str,
    presented: Option<&str>,
    variable_name: &str,
) -> Result<(), ApiError> {
    if expected.is_empty() {
        return Err(ApiError::Misconfigured(format!(
            "{variable_name} is not configured on the server"
        )));
    }

    let presented = presented.unwrap_or_default();
    // Hash-free constant-time compare: equal length AND equal bytes, both
    // folded into one `Choice` so neither short-circuits.
    let lengths_match = presented.len().ct_eq(&expected.len());
    let bytes_match = if presented.len() == expected.len() {
        presented.as_bytes().ct_eq(expected.as_bytes())
    } else {
        // Compare against itself to keep the work comparable, then discard.
        let _ = expected.as_bytes().ct_eq(expected.as_bytes());
        subtle::Choice::from(0u8)
    };

    if (lengths_match & bytes_match).into() {
        Ok(())
    } else {
        Err(ApiError::Unauthorized("Invalid secret".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_configured_secret() {
        assert!(verify_shared_secret("s3cret", Some("s3cret"), "TEST_SECRET").is_ok());
    }

    #[test]
    fn rejects_a_wrong_secret() {
        assert!(matches!(
            verify_shared_secret("s3cret", Some("wrong!"), "TEST_SECRET"),
            Err(ApiError::Unauthorized(_))
        ));
    }

    #[test]
    fn rejects_a_missing_secret_header() {
        assert!(matches!(
            verify_shared_secret("s3cret", None, "TEST_SECRET"),
            Err(ApiError::Unauthorized(_))
        ));
    }

    #[test]
    fn rejects_a_prefix_of_the_secret() {
        assert!(matches!(
            verify_shared_secret("s3cret", Some("s3c"), "TEST_SECRET"),
            Err(ApiError::Unauthorized(_))
        ));
    }

    #[test]
    fn an_unconfigured_secret_fails_closed() {
        assert!(matches!(
            verify_shared_secret("", Some("anything"), "TEST_SECRET"),
            Err(ApiError::Misconfigured(_))
        ));
        assert!(matches!(
            verify_shared_secret("", None, "TEST_SECRET"),
            Err(ApiError::Misconfigured(_))
        ));
    }
}
