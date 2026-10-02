//! Commands about the desktop app itself: what it can do, and notifications.

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::notify;
use crate::stores::{self, StoreAbilities};

/// What the web app needs to know to show the desktop buttons.
#[derive(Debug, Serialize)]
pub struct DesktopAbilities {
    /// The desktop app's version.
    pub version: String,
    /// What the app can do at each store.
    pub stores: Vec<StoreAbilities>,
}

/// Longest notification title and body accepted, in characters.
pub const MAX_TITLE: usize = 100;
pub const MAX_BODY: usize = 300;

/// What the desktop app can do here.
#[tauri::command]
pub fn desktop_abilities<R: Runtime>(app: AppHandle<R>) -> DesktopAbilities {
    DesktopAbilities {
        version: app.package_info().version.to_string(),
        stores: stores::ALL.iter().map(|store| store.abilities()).collect(),
    }
}

/// Shows a desktop notification.
#[tauri::command]
pub fn notify<R: Runtime>(app: AppHandle<R>, title: String, body: String) -> Result<(), String> {
    let (title, body) = checked_notice(&title, &body)?;
    notify::show(&app, &title, &body).map_err(|err| {
        log::error!("[notify] could not show a notification: {err}");
        "The notification could not be shown.".to_string()
    })
}

/// Trims the notice and refuses a blank or over-long one.
pub fn checked_notice(title: &str, body: &str) -> Result<(String, String), String> {
    let (title, body) = (title.trim(), body.trim());
    if title.is_empty() {
        return Err("A notification needs a title.".into());
    }
    if title.chars().count() > MAX_TITLE || body.chars().count() > MAX_BODY {
        return Err("The notification is too long.".into());
    }
    Ok((title.to_string(), body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_a_good_notice() {
        assert_eq!(
            checked_notice(" Trolley filled ", " 5 added "),
            Ok(("Trolley filled".into(), "5 added".into()))
        );
    }

    #[test]
    fn refuses_blank_and_long_notices() {
        assert!(checked_notice("  ", "body").is_err());
        assert!(checked_notice(&"t".repeat(MAX_TITLE + 1), "").is_err());
        assert!(checked_notice("title", &"b".repeat(MAX_BODY + 1)).is_err());
    }
}
