//! State shared by every request handler.

use std::sync::Arc;

use sqlx::PgPool;

use crate::auth::AuthProvider;
use crate::config::Settings;
use crate::services::encryption::Encryptor;
use crate::services::ws_hub::WsHub;

/// Handles to everything a route needs. Cheap to clone — the pool, hub and
/// provider are all shared references internally.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub settings: Arc<Settings>,
    pub hub: WsHub,
    pub auth: Arc<dyn AuthProvider>,
    /// Present only once `CREDENTIAL_ENCRYPTION_KEY` is set. Store credentials
    /// are not needed for voice intake, so a missing key is a warning at
    /// startup rather than a failure to boot.
    pub encryptor: Option<Encryptor>,
}
