//! One window per store. It keeps the webview's own cookies, which is how the
//! household stays logged in; the app never reads them.
//!
//! When a page finishes loading, a fill program waiting for this store's
//! trolley page (`PendingPrograms`) is run there.

use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use url::Url;

use crate::pending_program::PendingPrograms;
use crate::stores::Store;

/// Shows `store`'s window at `page`, making it the first time.
pub fn show_at<R: Runtime>(app: &AppHandle<R>, store: Store, page: &str) -> tauri::Result<()> {
    let url = Url::parse(page).expect("store pages are valid URLs");
    if let Some(window) = app.get_webview_window(store.window_label()) {
        window.navigate(url)?;
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, store.window_label(), WebviewUrl::External(url))
        .title(store.display_name())
        .inner_size(1100.0, 800.0)
        .on_page_load(move |webview, payload| {
            if payload.event() != PageLoadEvent::Finished {
                return;
            }
            let page = payload.url();
            log::info!(
                "[store-window] {store:?} loaded {}{}",
                page.host_str().unwrap_or(""),
                page.path()
            );
            let pending = webview.app_handle().state::<PendingPrograms>();
            if let Some(program) = pending.take_for(store, page) {
                log::info!("[store-window] running the fill program for {store:?}");
                if let Err(err) = webview.eval(&program) {
                    log::error!("[store-window] could not run the fill program: {err}");
                }
            }
        })
        .build()?;
    log::info!("[store-window] opened {store:?}");
    Ok(())
}
