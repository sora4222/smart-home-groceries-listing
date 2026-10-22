//! The app's own commands. Which page may call which is set by
//! `capabilities/` (setup page) and `server_access.rs` (server page).

pub mod desktop;
pub mod setup;
pub mod store;
