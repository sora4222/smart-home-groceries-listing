//! The menu bar / system tray icon. The app keeps running there when the main
//! window is closed, so notifications still arrive.
//!
//! Menu events arrive on the main thread. Actions that create a window run on
//! their own thread, because creating a webview inside an event handler
//! deadlocks on Windows (tauri-apps/wry#583).

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime};

use crate::stores::Store;
use crate::windows::main_window::{self, MainPage};
use crate::windows::store_window;

const OPEN: &str = "open";
const RELOAD: &str = "reload";
const WOOLWORTHS: &str = "woolworths";
const CHANGE_SERVER: &str = "change-server";
const QUIT: &str = "quit";

/// Adds the tray icon and its menu.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, OPEN, "Open Grocery List", true, None::<&str>)?,
            &MenuItem::with_id(app, RELOAD, "Reload server", true, None::<&str>)?,
            &MenuItem::with_id(app, WOOLWORTHS, "Open Woolworths", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                CHANGE_SERVER,
                "Change server address…",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Grocery List")
        .menu(&menu)
        .on_menu_event(|app, event| {
            if let Some(action) = TrayAction::from_id(event.id().as_ref()) {
                dispatch(app, action);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// One tray menu item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayAction {
    Open,
    Reload,
    Woolworths,
    ChangeServer,
    Quit,
}

impl TrayAction {
    fn from_id(id: &str) -> Option<Self> {
        match id {
            OPEN => Some(Self::Open),
            RELOAD => Some(Self::Reload),
            WOOLWORTHS => Some(Self::Woolworths),
            CHANGE_SERVER => Some(Self::ChangeServer),
            QUIT => Some(Self::Quit),
            _ => None,
        }
    }

    /// True when the action may build a new webview window.
    fn creates_window(self) -> bool {
        matches!(self, Self::Woolworths | Self::ChangeServer)
    }
}

/// Runs `action`, on its own thread when it may create a window.
fn dispatch<R: Runtime>(app: &AppHandle<R>, action: TrayAction) {
    if !action.creates_window() {
        run(app, action);
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || run(&app, action));
}

fn run<R: Runtime>(app: &AppHandle<R>, action: TrayAction) {
    log::info!("[tray] {action:?}");
    if let Err(err) = perform(app, action) {
        log::error!("[tray] {action:?} failed: {err}");
    }
}

fn perform<R: Runtime>(app: &AppHandle<R>, action: TrayAction) -> tauri::Result<()> {
    match action {
        TrayAction::Open => main_window::reveal(app),
        TrayAction::Reload => {
            if let Some(window) = app.get_webview_window(main_window::MAIN_LABEL) {
                window.reload()?;
            }
            Ok(())
        }
        TrayAction::Woolworths => {
            store_window::show_at(app, Store::Woolworths, Store::Woolworths.home_url())
        }
        TrayAction::ChangeServer => main_window::open(app, MainPage::Setup),
        TrayAction::Quit => {
            app.exit(0);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_menu_item() {
        assert_eq!(TrayAction::from_id(OPEN), Some(TrayAction::Open));
        assert_eq!(TrayAction::from_id(RELOAD), Some(TrayAction::Reload));
        assert_eq!(
            TrayAction::from_id(WOOLWORTHS),
            Some(TrayAction::Woolworths)
        );
        assert_eq!(
            TrayAction::from_id(CHANGE_SERVER),
            Some(TrayAction::ChangeServer)
        );
        assert_eq!(TrayAction::from_id(QUIT), Some(TrayAction::Quit));
        assert_eq!(TrayAction::from_id("other"), None);
    }

    #[test]
    fn only_window_creating_actions_leave_the_event_handler() {
        assert!(TrayAction::Woolworths.creates_window());
        assert!(TrayAction::ChangeServer.creates_window());
        assert!(!TrayAction::Open.creates_window());
        assert!(!TrayAction::Reload.creates_window());
        assert!(!TrayAction::Quit.creates_window());
    }
}
