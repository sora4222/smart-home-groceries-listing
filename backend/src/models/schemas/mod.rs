//! Request and response bodies, one module per API domain.
//!
//! Field names and constraints match the previous Pydantic schemas exactly,
//! so the existing web app needs no changes. Incoming bodies are validated
//! before they reach a service — length and range limits are enforced here,
//! not in SQL alone.
//!
//! Everything is re-exported flat, so callers import
//! `crate::models::schemas::GroceryItemCreate` without caring which file the
//! type lives in.

mod common;
mod grocery;
mod item_rules;
mod voice;

pub use common::*;
pub use grocery::*;
pub use item_rules::*;
pub use voice::*;
