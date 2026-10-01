//! Why a store search failed.
//!
//! Kept separate from [`crate::error::ApiError`]: a store failing is not a
//! failed request. The search answers 200 with that store marked unavailable,
//! and the other store's results still show.

/// A store search that produced no products.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    /// The store's bot protection refused the request (401, 403, 429 or a
    /// challenge page). Retrying immediately is what gets an account flagged,
    /// so this is reported rather than retried.
    #[error("the store refused the request (HTTP {status})")]
    Blocked { status: u16 },

    /// The store could not be reached, timed out, or answered 5xx.
    #[error("the store could not be reached: {0}")]
    Unreachable(String),

    /// The store answered, but not in the shape this application expects —
    /// usually the store changed its website.
    #[error("the store's response was not understood: {0}")]
    UnexpectedResponse(String),
}

impl StoreError {
    /// Short machine-readable kind, for logs and the API response.
    pub fn kind(&self) -> &'static str {
        match self {
            StoreError::Blocked { .. } => "blocked",
            StoreError::Unreachable(_) => "unreachable",
            StoreError::UnexpectedResponse(_) => "unexpected_response",
        }
    }

    /// A sentence the household can act on. Never includes upstream detail.
    pub fn user_message(&self, store_name: &str) -> String {
        match self {
            StoreError::Blocked { .. } => format!(
                "{store_name} refused the search (bot protection). Try again later."
            ),
            StoreError::Unreachable(_) => {
                format!("{store_name} could not be reached. Try again later.")
            }
            StoreError::UnexpectedResponse(_) => format!(
                "{store_name}'s website changed and its results could not be read."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_message_never_carries_upstream_detail() {
        let err = StoreError::Unreachable("dns error: secret-host.internal".into());
        let message = err.user_message("Coles");
        assert!(!message.contains("secret-host"));
        assert_eq!(err.kind(), "unreachable");
    }
}
