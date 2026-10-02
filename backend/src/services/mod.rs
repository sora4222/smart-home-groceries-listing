//! Business logic. Nothing here touches HTTP types, and nothing here creates
//! its own database pool — a service borrows what a route hands it.

pub mod encryption;
pub mod filter_terms;
pub mod grocery;
pub mod item_rules;
pub mod order_review;
pub mod product_search;
pub mod purchases;
pub mod selections;
pub mod spending;
pub mod stores;
pub mod triage;
pub mod trolley_handoffs;
pub mod voice;
pub mod voice_changes;
pub mod ws_hub;
