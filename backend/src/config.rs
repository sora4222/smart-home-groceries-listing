//! Runtime configuration, loaded from environment variables.
//!
//! Every value is sourced from the environment (see `.env.example` at the
//! repository root). Nothing here is hardcoded in application code, and no
//! secret in this struct is ever logged or returned to the frontend.

use std::env;
use std::time::Duration;

use crate::error::ConfigError;

/// How long a fetched Clerk JWKS document is reused before being re-fetched.
pub const JWKS_CACHE_TTL_SECONDS: u64 = 3600;

/// Configuration for the backend process.
#[derive(Debug, Clone)]
pub struct Settings {
    /// PostgreSQL connection string, e.g. `postgres://user:pw@host:5432/db`.
    pub database_url: String,
    /// Maximum pooled database connections.
    pub database_max_connections: u32,
    /// Address the HTTP server binds to.
    pub bind_address: String,

    /// Clerk JWKS document URL, used to verify session JWTs.
    pub clerk_jwks_url: String,

    /// Shared secret authenticating the generic intake webhook.
    pub voice_webhook_secret: String,
    /// Shared secret authenticating the Alexa bridge sidecar on loopback.
    pub alexa_bridge_secret: String,
    /// Shared secret the "Fill trolley" bookmarklet sends from the store's
    /// website (`STORE_TAB_SECRET`). It can only claim and report trolley
    /// handoffs. The web app reads it to build the bookmarklet.
    pub store_tab_secret: String,

    /// Base64-encoded 32-byte key for AES-256-GCM credential encryption.
    pub credential_encryption_key: String,

    /// Exact origins permitted by CORS. Never a wildcard.
    pub cors_origins: Vec<String>,

    /// When true, auth is bypassed and a fixed development user is injected.
    /// Never enable this on a host reachable from the internet.
    pub dev_auth_bypass: bool,

    /// How the backend reaches Woolworths and Coles.
    pub stores: StoreSettings,
}

/// Which store clients the backend uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    /// The real websites.
    Live,
    /// A small built-in catalogue; nothing leaves the machine.
    Fake,
}

/// Settings for the store integrations.
#[derive(Debug, Clone)]
pub struct StoreSettings {
    /// `STORE_CLIENTS=live|fake`. Defaults to `live`.
    pub mode: StoreMode,
    /// Overridable so tests can point a client at a mock server.
    pub woolworths_base_url: String,
    pub coles_base_url: String,
    /// How long one store may take to answer a search.
    pub timeout: Duration,
    /// How long a store's answer to a query is reused.
    pub cache_ttl: Duration,
}

impl StoreSettings {
    /// Reads the store settings from the environment.
    fn from_env() -> Result<Self, ConfigError> {
        let mode = match optional("STORE_CLIENTS").map(|v| v.trim().to_ascii_lowercase()) {
            None => StoreMode::Live,
            Some(value) if value == "live" => StoreMode::Live,
            Some(value) if value == "fake" => StoreMode::Fake,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    key: "STORE_CLIENTS".to_string(),
                })
            }
        };
        Ok(Self {
            mode,
            woolworths_base_url: optional("WOOLWORTHS_BASE_URL")
                .unwrap_or_else(|| "https://www.woolworths.com.au".to_string()),
            coles_base_url: optional("COLES_BASE_URL")
                .unwrap_or_else(|| "https://www.coles.com.au".to_string()),
            timeout: Duration::from_secs(parse_or("STORE_TIMEOUT_SECONDS", 12)?),
            cache_ttl: Duration::from_secs(parse_or("STORE_SEARCH_CACHE_SECONDS", 600)?),
        })
    }
}

impl Settings {
    /// Reads settings from the process environment, applying defaults that are
    /// safe for local development.
    ///
    /// Fails only when a value is present but unusable (for example a
    /// non-numeric port), so a misconfiguration surfaces at startup rather
    /// than on the first request that needs it.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: optional("DATABASE_URL").unwrap_or_else(|| {
                "postgres://grocery:changeme@localhost:5432/grocery".to_string()
            }),
            database_max_connections: parse_or("DATABASE_MAX_CONNECTIONS", 10)?,
            bind_address: optional("BIND_ADDRESS").unwrap_or_else(|| "0.0.0.0:8000".to_string()),

            clerk_jwks_url: optional("CLERK_JWKS_URL").unwrap_or_default(),

            voice_webhook_secret: optional("VOICE_WEBHOOK_SECRET").unwrap_or_default(),
            alexa_bridge_secret: optional("ALEXA_BRIDGE_SECRET").unwrap_or_default(),
            store_tab_secret: optional("STORE_TAB_SECRET").unwrap_or_default(),

            credential_encryption_key: optional("CREDENTIAL_ENCRYPTION_KEY").unwrap_or_default(),

            cors_origins: parse_csv(
                &optional("CORS_ORIGINS").unwrap_or_else(|| "http://localhost:3000".to_string()),
            ),

            dev_auth_bypass: parse_bool("DEV_AUTH_BYPASS"),

            stores: StoreSettings::from_env()?,
        })
    }
}

/// Returns an environment variable, treating blank values as absent.
fn optional(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

/// Parses a numeric environment variable, falling back to `default` when unset.
fn parse_or<T>(key: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    match optional(key) {
        None => Ok(default),
        Some(raw) => raw.trim().parse::<T>().map_err(|_| ConfigError::Invalid {
            key: key.to_string(),
        }),
    }
}

/// Parses a boolean environment variable; anything other than a recognised
/// truthy value is false, so a typo fails closed.
fn parse_bool(key: &str) -> bool {
    matches!(
        optional(key)
            .map(|v| v.trim().to_ascii_lowercase())
            .as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

/// Splits a comma-separated list, discarding blank entries.
fn parse_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_trims_and_drops_blanks() {
        assert_eq!(
            parse_csv("http://a, http://b ,, "),
            vec!["http://a".to_string(), "http://b".to_string()]
        );
    }

    #[test]
    fn parse_csv_of_empty_string_is_empty() {
        assert!(parse_csv("   ").is_empty());
    }
}
