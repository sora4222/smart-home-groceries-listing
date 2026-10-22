//! One window per store. It keeps the webview's own cookies, which is how the
//! household stays logged in; the app never reads them.
//!
//! When the trolley page finishes loading and a fill is waiting
//! (`FillOperations`), the program starts there and `fill_watch` follows it.
//! Closing the window ends that fill.

use tauri::webview::PageLoadEvent;
use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use url::Url;

use crate::fill_operation::FillOperations;
use crate::fill_watch;
use crate::stores::{Store, StorePage};

/// Shows `store`'s window at `page`, making it the first time.
pub fn show_at<R: Runtime>(app: &AppHandle<R>, store: Store, page: StorePage) -> tauri::Result<()> {
    let url = Url::parse(store.url(page)).expect("store pages are valid URLs");
    if let Some(window) = app.get_webview_window(store.window_label()) {
        window.navigate(url)?;
        return bring_forward(&window);
    }
    let window = WebviewWindowBuilder::new(app, store.window_label(), WebviewUrl::External(url))
        .title(store.display_name())
        .inner_size(1100.0, 800.0)
        .on_page_load(move |webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                start_waiting_fill(&webview, store, payload.url());
            }
        })
        .build()?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            let closed = Err(fill_watch::WINDOW_CLOSED.into());
            handle.state::<FillOperations>().finish(store, closed);
        }
    });
    log::info!("[store-window] opened {store:?}");
    Ok(())
}

/// Brings `store`'s window forward without changing its page. False when it
/// is not open.
pub fn reveal<R: Runtime>(app: &AppHandle<R>, store: Store) -> tauri::Result<bool> {
    match app.get_webview_window(store.window_label()) {
        Some(window) => bring_forward(&window).map(|()| true),
        None => Ok(false),
    }
}

fn bring_forward<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

/// Starts the waiting fill program when `page` is the trolley page.
fn start_waiting_fill<R: Runtime>(webview: &WebviewWindow<R>, store: Store, page: &Url) {
    log::info!(
        "[store-window] {store:?} loaded {}{}",
        page.host_str().unwrap_or(""),
        page.path()
    );
    let app = webview.app_handle();
    let operations = app.state::<FillOperations>();
    let Some(program) = operations.start_on(store, page) else {
        return;
    };
    log::info!("[store-window] running the fill program for {store:?}");
    if let Err(err) = webview.eval(&program) {
        log::error!("[store-window] could not run the fill program: {err}");
        operations.finish(
            store,
            Err("The fill could not start in the store window.".into()),
        );
        return;
    }
    tauri::async_runtime::spawn(fill_watch::watch(app.clone(), store));
}
