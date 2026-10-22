//! The menu bar / system tray icon. The app keeps running there when the main
//! window is closed, so notifications still arrive.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Runtime};

use crate::stores::Store;
use crate::windows::main_window::{self, MainPage};
use crate::windows::store_window;

const OPEN: &str = "open";
const WOOLWORTHS: &str = "woolworths";
const CHANGE_SERVER: &str = "change-server";
const QUIT: &str = "quit";

/// Adds the tray icon and its menu.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, OPEN, "Open Grocery List", true, None::<&str>)?,
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
            if let Err(err) = on_menu(app, event.id().as_ref()) {
                log::error!("[tray] {:?} failed: {err}", event.id());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn on_menu<R: Runtime>(app: &AppHandle<R>, id: &str) -> tauri::Result<()> {
    log::info!("[tray] {id}");
    match id {
        OPEN => main_window::reveal(app),
        WOOLWORTHS => store_window::show_at(app, Store::Woolworths, Store::Woolworths.home_url()),
        CHANGE_SERVER => main_window::open(app, MainPage::Setup),
        QUIT => {
            app.exit(0);
            Ok(())
        }
        _ => Ok(()),
    }
}
