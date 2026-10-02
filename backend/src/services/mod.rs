//! Business logic. Nothing here touches HTTP types, and nothing here creates
//! its own database pool — a service borrows what a route hands it.

pub mod dislikes;
pub mod encryption;
pub mod filter_terms;
pub mod grocery;
pub mod item_rules;
pub mod order_review;
pub mod product_search;
pub mod selections;
pub mod stores;
pub mod triage;
pub mod trolley_handoffs;
pub mod voice;
pub mod ws_hub;
