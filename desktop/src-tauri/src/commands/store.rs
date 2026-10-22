//! Commands that act at a store, in that store's window.
//!
//! They are `async` so Tauri runs them off the main thread: creating a webview
//! from a synchronous command deadlocks on Windows (tauri-apps/wry#583).

use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

use crate::fill_failure::FillFailure;
use crate::fill_operation::{FillOperations, FillReport};
use crate::stores::{Store, StorePage};
use crate::windows::store_window;

/// Longest fill program accepted, in bytes. The real one is about 20 kB.
pub const MAX_PROGRAM_BYTES: usize = 512 * 1024;

/// Longest a fill may take, including logging in on the way to the trolley.
const FILL_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Shows the store's website, to log in.
#[tauri::command]
pub async fn open_store<R: Runtime>(app: AppHandle<R>, store: Store) -> Result<(), String> {
    show_page(&app, store, StorePage::Home)
}

/// Shows the store's trolley page, runs `program` there once it loads, and
/// answers with the program's result.
///
/// `program` is the fill program the bookmark carries
/// (`frontend/src/lib/store-tab/bookmarklet.ts`). It also reports to the
/// server itself. One fill runs per store; the operation ends however this
/// command ends (`fill_operation.rs`).
#[tauri::command]
pub async fn fill_store_trolley<R: Runtime>(
    app: AppHandle<R>,
    store: Store,
    program: String,
) -> Result<FillReport, String> {
    if !store.abilities().allows(StorePage::Trolley) {
        return Err(not_yet(store));
    }
    check_program(&program)?;
    let size = program.len();
    let ticket = app
        .state::<FillOperations>()
        .inner()
        .begin(store, program)
        .map_err(|failure| failure.to_string())?;
    log::info!("[store] fill program for {store:?} waiting for the trolley page ({size} bytes)");
    store_window::show_at(&app, store, StorePage::Trolley).map_err(window_failed)?;
    // The IPC boundary: the web app reads a failure as its sentence.
    tokio::time::timeout(FILL_TIMEOUT, ticket.outcome())
        .await
        .unwrap_or(Err(FillFailure::TimedOut))
        .map_err(|failure| failure.to_string())
}

/// Shows the store's checkout page, to check and pay.
#[tauri::command]
pub async fn open_store_checkout<R: Runtime>(
    app: AppHandle<R>,
    store: Store,
) -> Result<(), String> {
    show_page(&app, store, StorePage::Checkout)
}

/// Shows `page` in `store`'s window when the app may show it there, unless
/// a fill is running in that window.
fn show_page<R: Runtime>(app: &AppHandle<R>, store: Store, page: StorePage) -> Result<(), String> {
    if !store.abilities().allows(page) {
        return Err(not_yet(store));
    }
    refuse_while_filling(app, store)?;
    log::info!("[store] opening {store:?} at {page:?}");
    store_window::show_at(app, store, page).map_err(window_failed)
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

/// Moving the store window would interrupt a running fill. A waiting fill
/// is left alone: logging in on the way to the trolley is expected.
fn refuse_while_filling<R: Runtime>(app: &AppHandle<R>, store: Store) -> Result<(), String> {
    if app.state::<FillOperations>().is_running(store) {
        return Err(format!(
            "Wait for the trolley fill at {} to finish.",
            store.display_name()
        ));
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

    /// Compiles only for commands that return a future, which Tauri runs off
    /// the main thread. Synchronous window creation deadlocks on Windows.
    fn runs_off_the_main_thread<F: std::future::Future>(_: F) {}

    #[test]
    fn window_commands_are_async() {
        let app = tauri::test::mock_app();
        let handle = app.handle().clone();
        runs_off_the_main_thread(open_store(handle.clone(), Store::Coles));
        runs_off_the_main_thread(fill_store_trolley(handle.clone(), Store::Coles, "x".into()));
        runs_off_the_main_thread(open_store_checkout(handle, Store::Coles));
    }

    #[test]
    fn says_which_store_is_not_ready() {
        assert_eq!(
            not_yet(Store::Coles),
            "The desktop app cannot do this at Coles yet."
        );
    }
}
