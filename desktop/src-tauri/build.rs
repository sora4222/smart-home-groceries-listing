//! Build script: generates Tauri's context and one permission per app command.
//!
//! Listing the commands makes Tauri deny every command not granted by a
//! capability, so the server page and the setup page only reach what
//! `capabilities/` and `server_access.rs` give them.

/// The app's own commands. Each gets an `allow-<name>` permission.
const COMMANDS: &[&str] = &[
    "desktop_abilities",
    "open_store",
    "fill_store_trolley",
    "open_store_checkout",
    "notify",
    "save_server_url",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run the Tauri build script");
}
