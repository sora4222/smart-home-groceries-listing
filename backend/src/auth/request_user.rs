//! Who a request signed in as, passed from the auth check to the access log.
//!
//! The access-log middleware runs around every request but cannot know the
//! user until a route's [`AuthUser`](super::AuthUser) extractor has verified
//! the token. Verifying the token a second time in the middleware would double
//! the work, so the middleware puts an empty [`RequestUser`] in the request's
//! extensions, the extractor fills it on success, and the middleware reads it
//! once the response is ready.

use std::sync::{Arc, OnceLock};

/// A slot for the signed-in user's id, shared between the middleware and the
/// auth extractor of one request. Cheap to clone.
#[derive(Debug, Clone, Default)]
pub struct RequestUser(Arc<OnceLock<String>>);

impl RequestUser {
    /// Records the user this request signed in as. Only the first call counts:
    /// a request is one user.
    pub fn record(&self, user_id: &str) {
        let _ = self.0.set(user_id.to_string());
    }

    /// The signed-in user's id, or `None` when no route verified one.
    pub fn id(&self) -> Option<&str> {
        self.0.get().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::RequestUser;

    #[test]
    fn is_empty_until_recorded() {
        assert_eq!(RequestUser::default().id(), None);
    }

    #[test]
    fn a_clone_sees_what_the_original_recorded() {
        let slot = RequestUser::default();
        let seen_by_middleware = slot.clone();

        slot.record("user_123");

        assert_eq!(seen_by_middleware.id(), Some("user_123"));
    }

    #[test]
    fn the_first_recorded_user_wins() {
        let slot = RequestUser::default();
        slot.record("user_1");
        slot.record("user_2");
        assert_eq!(slot.id(), Some("user_1"));
    }
}
