//! Business logic. Nothing here touches HTTP types, and nothing here creates
//! its own database pool — a service borrows what a route hands it.

pub mod encryption;
pub mod filter_terms;
pub mod grocery;
pub mod voice;
pub mod ws_hub;
