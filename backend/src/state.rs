//! State shared by every request handler.

use std::sync::Arc;

use sqlx::PgPool;

use crate::auth::AuthProvider;
use crate::config::Settings;
use crate::services::encryption::Encryptor;
use crate::services::google_tasks::GoogleTasks;
use crate::services::stores::StoreClients;
use crate::services::triage::TriageQueue;
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
    /// Woolworths and Coles, real or fake as `STORE_CLIENTS` chose.
    pub stores: StoreClients,
    /// Classifies intake requests in the background, as
    /// `INTAKE_LLM_PROVIDER` chose.
    pub triage: TriageQueue,
    /// The Google Tasks client, real or fake as `GOOGLE_TASKS_CLIENT` chose.
    pub google_tasks: GoogleTasks,
}
