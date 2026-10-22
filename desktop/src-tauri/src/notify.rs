//! Desktop notifications, through the notification plugin.

use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

/// Shows one notification.
pub fn show<R: Runtime>(
    app: &AppHandle<R>,
    title: &str,
    body: &str,
) -> tauri_plugin_notification::Result<()> {
    log::info!("[notify] {title}");
    app.notification().builder().title(title).body(body).show()
}
