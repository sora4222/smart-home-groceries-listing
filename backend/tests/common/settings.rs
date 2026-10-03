//! The settings every integration test runs with: auth bypassed, known
//! secrets, fake stores, fake Google Tasks, triage off, and no external calls.

use grocery_backend::config::{
    GoogleTasksMode, GoogleTasksSettings, Settings, StoreMode, StoreSettings, TriageProvider,
    TriageSettings,
};

use super::{TEST_BRIDGE_SECRET, TEST_STORE_TAB_SECRET, TEST_WEBHOOK_SECRET};

/// Settings for a test run: auth bypassed, known secrets, fake stores, no
/// external calls.
pub fn test_settings() -> Settings {
    Settings {
        // The pool is supplied directly, so this is never dialled.
        database_url: String::new(),
        database_max_connections: 5,
        bind_address: "127.0.0.1:0".to_string(),
        clerk_jwks_url: String::new(),
        voice_webhook_secret: TEST_WEBHOOK_SECRET.to_string(),
        alexa_bridge_secret: TEST_BRIDGE_SECRET.to_string(),
        store_tab_secret: TEST_STORE_TAB_SECRET.to_string(),
        credential_encryption_key: String::new(),
        cors_origins: vec!["http://localhost:3000".to_string()],
        dev_auth_bypass: true,
        stores: fake_store_settings(),
        triage: triage_settings(TriageProvider::Off, ""),
        google_tasks: google_tasks_settings(GoogleTasksMode::Fake, "http://127.0.0.1:9"),
    }
}

/// A base64 AES-256 key for tests that store a Google sign-in. Test-only.
pub const TEST_ENCRYPTION_KEY: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";

/// Google Tasks settings for a test: the fake account, or the live client
/// aimed at a `wiremock` server at `base_url`. Never polls on its own.
pub fn google_tasks_settings(mode: GoogleTasksMode, base_url: &str) -> GoogleTasksSettings {
    GoogleTasksSettings {
        mode,
        client_id: "test-client-id".to_string(),
        client_secret: "test-client-secret".to_string(),
        redirect_uri: "http://localhost:3000/settings/intake".to_string(),
        auth_url: format!("{base_url}/o/oauth2/v2/auth"),
        token_url: format!("{base_url}/token"),
        api_base: base_url.to_string(),
        timeout: std::time::Duration::from_secs(2),
        poll_in_background: false,
    }
}

/// Triage settings for a test. Off by default, so a request is in Pending
/// Requests the moment it is recorded; the triage tests switch it on.
pub fn triage_settings(provider: TriageProvider, base_url: &str) -> TriageSettings {
    TriageSettings {
        provider,
        base_url: base_url.to_string(),
        model: "test-model".to_string(),
        api_key: "test-key".to_string(),
        timeout: std::time::Duration::from_secs(2),
        min_confidence: 0.7,
    }
}

/// The fake catalogue: an integration test must never reach a real store.
pub fn fake_store_settings() -> StoreSettings {
    StoreSettings {
        mode: StoreMode::Fake,
        woolworths_base_url: "http://127.0.0.1:9".to_string(),
        coles_base_url: "http://127.0.0.1:8".to_string(),
        timeout: std::time::Duration::from_secs(2),
        cache_ttl: std::time::Duration::from_secs(60),
    }
}
