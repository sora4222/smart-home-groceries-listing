//! One-use `state` values for the Google sign-in round trip.
//!
//! The sign-in starts in a signed-in browser session and comes back as a page
//! load carrying `?code=...&state=...`. The state proves the reply belongs to
//! a sign-in this app started (no cross-site request forgery). Only its
//! SHA-256 hash is stored, it lasts ten minutes, and it is deleted when used.

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;

/// How long someone has to finish signing in with Google.
const LIFETIME_MINUTES: i64 = 10;

/// A new random state: two v4 UUIDs, 244 random bits.
pub fn new_state() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// The hex SHA-256 of a state, as stored.
fn hash(state: &str) -> String {
    Sha256::digest(state.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Remembers a state for [`LIFETIME_MINUTES`], dropping any that expired.
pub async fn remember<'e, E>(executor: E, state: &str) -> Result<(), ApiError>
where
    E: PgExecutor<'e> + Copy,
{
    sqlx::query("DELETE FROM google_oauth_states WHERE expires_at < now()")
        .execute(executor)
        .await?;
    sqlx::query("INSERT INTO google_oauth_states (id, state_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(hash(state))
        .bind(Utc::now() + Duration::minutes(LIFETIME_MINUTES))
        .execute(executor)
        .await?;
    Ok(())
}

/// Uses up a state. True only for a state this app issued, still in date and
/// never used before.
pub async fn take<'e, E>(executor: E, state: &str) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let taken: Option<Uuid> = sqlx::query_scalar(
        "DELETE FROM google_oauth_states WHERE state_hash = $1 AND expires_at > now()
         RETURNING id",
    )
    .bind(hash(state))
    .fetch_optional(executor)
    .await?;
    Ok(taken.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_are_long_random_and_hash_to_64_hex_characters() {
        let (a, b) = (new_state(), new_state());
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert_eq!(hash(&a).len(), 64);
        assert_ne!(hash(&a), a);
    }
}
