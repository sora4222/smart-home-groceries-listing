//! Row types for `product_dislikes` and `dislike_overrides`.
//!
//! Kept in their own file so `db.rs` stays short; re-exported from
//! `models::db` like every other row.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::services::stores::Store;

/// One household member's dislike of one store product.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct ProductDislike {
    pub id: Uuid,
    /// The member's id from the auth provider.
    pub user_id: String,
    /// How other members see them on the badge.
    pub user_name: String,
    pub store: Store,
    /// The store's own id for the product.
    pub product_id: String,
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub disliked_at: DateTime<Utc>,
}

/// "Buy it this time anyway": a dislike set aside for one list item.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct DislikeOverride {
    pub id: Uuid,
    pub grocery_item_id: Uuid,
    pub store: Store,
    pub product_id: String,
    /// The member who chose to buy it anyway.
    pub overridden_by: String,
    pub overridden_at: DateTime<Utc>,
}
