//! Starts the desktop app. Everything else lives in the library.

// Without this Windows opens a console window next to the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    grocery_desktop_lib::run();
}
