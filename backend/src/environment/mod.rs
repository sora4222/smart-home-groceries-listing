//! Startup configuration sources, independent of settings and business logic.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod onepassword;

/// Load selected dotenv keys with exported overrides and a replaceable resolver.
/// Errors name the source or key and never include its value.
pub fn load_environment(
    file: Option<&Path>,
    inherited: &HashMap<String, String>,
    keys: &[&str],
    mut resolve: impl FnMut(&str) -> anyhow::Result<String>,
) -> anyhow::Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    if let Some(file) = file {
        let entries = dotenvy::from_path_iter(file)
            .map_err(|_| anyhow::anyhow!("Cannot read environment file"))?;
        for entry in entries {
            let (key, value) =
                entry.map_err(|_| anyhow::anyhow!("Cannot parse environment file"))?;
            if keys.contains(&key.as_str()) {
                values.insert(key, value);
            }
        }
    }
    for &key in keys {
        if let Some(value) = inherited.get(key) {
            values.insert(key.to_string(), value.clone());
        }
    }
    for (key, value) in &mut values {
        if value.starts_with("op://") {
            *value = resolve(value).map_err(|_| {
                anyhow::anyhow!(
                    "Cannot load {key} from 1Password; check CLI access and the reference"
                )
            })?;
            anyhow::ensure!(
                !value.is_empty() && !value.starts_with("op://"),
                "1Password returned no resolved value for {key}"
            );
        }
    }
    Ok(values)
}

/// Load backend configuration before creating threads or initializing logging.
/// Call only from the synchronous process entrypoint.
pub fn initialize() -> anyhow::Result<()> {
    let inherited = std::env::vars().collect();
    let file = env_file()?;
    let values = load_environment(file.as_deref(), &inherited, KEYS, onepassword::read)?;
    for (key, value) in values {
        std::env::set_var(key, value);
    }
    Ok(())
}

/// Find the nearest dotenv file, or use an explicitly selected worktree file.
fn env_file() -> anyhow::Result<Option<PathBuf>> {
    if let Some(file) = std::env::var_os("GROCERY_ENV_FILE") {
        return Ok(Some(PathBuf::from(file)));
    }
    let current = std::env::current_dir()?;
    Ok(current
        .ancestors()
        .map(|dir| dir.join(".env"))
        .find(|file| file.is_file()))
}

// Only settings used by the backend; frontend and tunnel failures cannot block it.
const KEYS: &[&str] = &[
    "DATABASE_URL",
    "DATABASE_MAX_CONNECTIONS",
    "POSTGRES_USER",
    "POSTGRES_PASSWORD",
    "BIND_ADDRESS",
    "CLERK_JWKS_URL",
    "VOICE_WEBHOOK_SECRET",
    "ALEXA_BRIDGE_SECRET",
    "STORE_TAB_SECRET",
    "CREDENTIAL_ENCRYPTION_KEY",
    "CORS_ORIGINS",
    "DEV_AUTH_BYPASS",
    "STORE_CLIENTS",
    "WOOLWORTHS_BASE_URL",
    "COLES_BASE_URL",
    "STORE_TIMEOUT_SECONDS",
    "STORE_SEARCH_CACHE_SECONDS",
    "INTAKE_LLM_PROVIDER",
    "INTAKE_LLM_BASE_URL",
    "INTAKE_LLM_MODEL",
    "INTAKE_LLM_API_KEY",
    "OPENAI_API_KEY",
    "INTAKE_LLM_MIN_CONFIDENCE",
    "INTAKE_LLM_TIMEOUT_SECONDS",
    "GOOGLE_TASKS_CLIENT",
    "GOOGLE_CLIENT_ID",
    "GOOGLE_CLIENT_SECRET",
    "GOOGLE_REDIRECT_URI",
    "GOOGLE_OAUTH_AUTH_URL",
    "GOOGLE_OAUTH_TOKEN_URL",
    "GOOGLE_TASKS_API_BASE",
    "GOOGLE_TIMEOUT_SECONDS",
    "GOOGLE_TASKS_NO_BACKGROUND_POLL",
    "ACCESS_LOG_DIR",
    "RUST_LOG",
];
