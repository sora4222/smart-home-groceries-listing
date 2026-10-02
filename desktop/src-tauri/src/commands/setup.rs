//! The setup page's one command: save the server address.

use tauri::{AppHandle, Manager, Runtime};

use crate::settings::{self, Settings};

/// Saves the server address and restarts the app, which then opens the
/// server and grants it the app's commands. Answers with a plain sentence
/// when the address is refused.
#[tauri::command]
pub fn save_server_url<R: Runtime>(app: AppHandle<R>, url: String) -> Result<(), String> {
    let origin = settings::parse_server_url(&url).map_err(|problem| problem.to_string())?;
    let dir = app.path().app_config_dir().map_err(|err| {
        log::error!("[setup] no config folder: {err}");
        "The app has no folder to save settings in.".to_string()
    })?;
    settings::save(
        &dir,
        &Settings {
            server_url: Some(origin),
        },
    )
    .map_err(|err| {
        log::error!("[setup] could not save settings: {err}");
        "The address could not be saved.".to_string()
    })?;
    log::info!("[setup] restarting to open the new server address");
    app.restart()
}
