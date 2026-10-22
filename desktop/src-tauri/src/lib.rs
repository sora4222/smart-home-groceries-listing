//! The grocery desktop app: shows the server's web app and does the
//! Woolworths (later Coles) steps on this computer.
//! Spec: `docs/features/FEATURE_DESKTOP_APP.md`.

mod commands;
mod fill_operation;
mod fill_status;
mod fill_watch;
mod notify;
mod server_access;
mod settings;
mod startup;
mod stores;
mod tray;
mod windows;

use tauri::WindowEvent;

use fill_operation::FillOperations;
use windows::main_window::{self, MAIN_LABEL};

/// Builds and runs the app.
pub fn run() {
    tauri::Builder::default()
        // First, so a second launch only brings the open app forward.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("[app] second launch; showing the open window");
            let _ = main_window::reveal(app);
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .manage(FillOperations::default())
        .invoke_handler(tauri::generate_handler![
            commands::desktop::desktop_abilities,
            commands::desktop::notify,
            commands::store::open_store,
            commands::store::fill_store_trolley,
            commands::store::open_store_checkout,
            commands::setup::save_server_url,
        ])
        .setup(|app| {
            let handle = app.handle();
            main_window::open(handle, startup::first_page(handle))?;
            tray::build(handle)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window hides it; the app stays in the tray.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == MAIN_LABEL {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("the desktop app failed to start");
}
