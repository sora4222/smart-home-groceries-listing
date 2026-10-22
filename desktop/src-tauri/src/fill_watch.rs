//! Follows a started fill program on the store page until it reports, then
//! finishes its operation (`fill_operation.rs`) with the result.

use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use tokio::sync::oneshot;

use crate::fill_operation::{FillFailure, FillOperations, FillReport};
use crate::fill_status::{self, PageStatus};
use crate::stores::Store;

/// How often the store page is asked for the program's progress.
const POLL_EVERY: Duration = Duration::from_secs(1);
/// How long one read may take before it is tried again.
const READ_WITHIN: Duration = Duration::from_secs(5);

/// Reads progress until the program reports or the operation ends elsewhere.
pub async fn watch<R: Runtime>(app: AppHandle<R>, store: Store) {
    let operations = app.state::<FillOperations>();
    while operations.is_running(store) {
        tokio::time::sleep(POLL_EVERY).await;
        let Some(window) = app.get_webview_window(store.window_label()) else {
            operations.finish(store, Err(FillFailure::WindowClosed));
            return;
        };
        let status = match read(&window).await {
            Some(status) => status,
            None => continue,
        };
        match status {
            PageStatus::Running => {}
            PageStatus::Completed(message) => operations.finish(store, Ok(FillReport { message })),
            PageStatus::Failed(message) => {
                operations.finish(store, Err(FillFailure::Program(message)))
            }
            PageStatus::Gone => operations.finish(store, Err(FillFailure::PageChanged)),
        }
    }
}

/// One progress read, or `None` when the page did not answer in time.
async fn read<R: Runtime>(window: &WebviewWindow<R>) -> Option<PageStatus> {
    let (sender, answer) = oneshot::channel();
    let sender = Mutex::new(Some(sender));
    let asked = window.eval_with_callback(fill_status::READ_STATUS, move |text| {
        if let Some(sender) = sender.lock().ok().and_then(|mut slot| slot.take()) {
            let _ = sender.send(text);
        }
    });
    if let Err(err) = asked {
        log::warn!("[fill] could not read the store page: {err}");
        return None;
    }
    match tokio::time::timeout(READ_WITHIN, answer).await {
        Ok(Ok(text)) => Some(fill_status::parse(&text)),
        _ => None,
    }
}
