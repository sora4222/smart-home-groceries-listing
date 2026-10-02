//! Commands that act at a store, in that store's window.

use tauri::{AppHandle, Manager, Runtime};

use crate::pending_program::PendingPrograms;
use crate::stores::Store;
use crate::windows::store_window;

/// Longest fill program accepted, in bytes. The real one is about 20 kB.
pub const MAX_PROGRAM_BYTES: usize = 512 * 1024;

/// Shows the store's website, to log in.
#[tauri::command]
pub fn open_store<R: Runtime>(app: AppHandle<R>, store: Store) -> Result<(), String> {
    if !store.abilities().login {
        return Err(not_yet(store));
    }
    log::info!("[store] opening {store:?} to log in");
    store_window::show_at(&app, store, store.home_url()).map_err(window_failed)
}

/// Shows the store's trolley page and runs `program` there once it loads.
///
/// `program` is the fill program the bookmark carries
/// (`frontend/src/lib/store-tab/bookmarklet.ts`). It reports to the server
/// itself; the web app follows the handoff and notifies when it is done.
#[tauri::command]
pub fn fill_store_trolley<R: Runtime>(
    app: AppHandle<R>,
    store: Store,
    program: String,
) -> Result<(), String> {
    if !store.abilities().fill_trolley {
        return Err(not_yet(store));
    }
    check_program(&program)?;
    log::info!(
        "[store] fill program for {store:?} waiting for the trolley page ({} bytes)",
        program.len()
    );
    app.state::<PendingPrograms>().set(store, program);
    store_window::show_at(&app, store, store.trolley_url()).map_err(window_failed)
}

/// Shows the store's checkout page, to check and pay.
#[tauri::command]
pub fn open_store_checkout<R: Runtime>(app: AppHandle<R>, store: Store) -> Result<(), String> {
    if !store.abilities().checkout {
        return Err(not_yet(store));
    }
    log::info!("[store] opening {store:?} checkout");
    store_window::show_at(&app, store, store.checkout_url()).map_err(window_failed)
}

/// Refuses an empty or oversized program.
pub fn check_program(program: &str) -> Result<(), String> {
    if program.trim().is_empty() {
        return Err("The fill program is empty.".into());
    }
    if program.len() > MAX_PROGRAM_BYTES {
        return Err("The fill program is too large.".into());
    }
    Ok(())
}

fn not_yet(store: Store) -> String {
    format!(
        "The desktop app cannot do this at {} yet.",
        store.display_name()
    )
}

fn window_failed(err: tauri::Error) -> String {
    log::error!("[store] store window failed: {err}");
    "The store window could not be opened.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_empty_and_oversized_programs() {
        assert!(check_program("  ").is_err());
        assert!(check_program(&"x".repeat(MAX_PROGRAM_BYTES + 1)).is_err());
        assert!(check_program("(function(){})()").is_ok());
    }

    #[test]
    fn says_which_store_is_not_ready() {
        assert_eq!(
            not_yet(Store::Coles),
            "The desktop app cannot do this at Coles yet."
        );
    }
}
