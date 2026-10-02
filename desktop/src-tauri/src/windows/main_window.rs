//! The main window: the server's web app, or the setup page when no server
//! address is saved yet.

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use url::Url;

/// The main window's label. Capabilities name it.
pub const MAIN_LABEL: &str = "main";

/// The bundled setup page (`desktop/setup/index.html`).
const SETUP_PAGE: &str = "index.html";

/// Where the main window should go.
pub enum MainPage {
    /// The server's web app at this origin.
    Server(Url),
    /// The setup page, to type the server address.
    Setup,
}

/// Opens the main window at `page`. An open main window is replaced, so the
/// page always starts fresh (the setup page has no server state to keep).
pub fn open<R: Runtime>(app: &AppHandle<R>, page: MainPage) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        window.destroy()?;
    }
    let url = match page {
        MainPage::Server(url) => WebviewUrl::External(url),
        MainPage::Setup => WebviewUrl::App(SETUP_PAGE.into()),
    };
    log::info!("[main-window] opening {url}");
    WebviewWindowBuilder::new(app, MAIN_LABEL, url)
        .title("Grocery List")
        .inner_size(1100.0, 800.0)
        .min_inner_size(390.0, 600.0)
        .build()?;
    Ok(())
}

/// Brings the main window back (from the tray, or a second launch).
pub fn reveal<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}
