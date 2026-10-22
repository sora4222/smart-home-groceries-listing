//! Which page the main window opens first: the saved server address, then
//! the launch value, then the build default (`settings::starting_url`). The
//! chosen server is granted the app's commands before it loads.

use tauri::{AppHandle, Manager, Runtime};

use crate::server_access;
use crate::settings::{self, ServerOrigin};
use crate::windows::main_window::MainPage;

/// The server's web app when an address is known, else the setup page.
pub fn first_page<R: Runtime>(app: &AppHandle<R>) -> MainPage {
    let saved = app
        .path()
        .app_config_dir()
        .map(|dir| settings::load(&dir))
        .unwrap_or_default();
    let runtime = std::env::var("GROCERY_DESKTOP_SERVER_URL").ok();
    match settings::starting_url(
        &saved,
        runtime.as_deref(),
        option_env!("GROCERY_DESKTOP_SERVER_URL"),
    ) {
        Ok(Some(origin)) => server_page(app, origin),
        Ok(None) => MainPage::Setup,
        Err(problem) => {
            log::warn!("[app] server address refused: {problem}");
            MainPage::Setup
        }
    }
}

/// Grants `origin` the app's commands and opens it; setup when that fails.
fn server_page<R: Runtime>(app: &AppHandle<R>, origin: ServerOrigin) -> MainPage {
    if let Err(err) = server_access::grant(app, origin.as_str()) {
        log::error!(
            "[app] could not grant {} the app's commands: {err}",
            origin.as_str()
        );
        return MainPage::Setup;
    }
    log::info!("[app] opening {}", origin.as_str());
    MainPage::Server(origin.into_url())
}
