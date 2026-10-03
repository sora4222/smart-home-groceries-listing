//! Bodies for product dislikes and for overriding one for an order.
//!
//! A dislike names the store's product and carries the label the price
//! comparison showed, so the household list can say what was disliked
//! without searching the store again. Each response says whether the dislike
//! is the caller's own (`mine`), since only that one can be removed.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::db::{DislikeOverride, ProductDislike};
use crate::models::schemas::MAX_PRODUCT_ID_LEN;
use crate::services::stores::Store;

/// Longest product name accepted, matching the `product_name` CHECK.
pub const MAX_PRODUCT_NAME_LEN: u64 = 300;
/// Longest brand accepted, matching the `brand` CHECK.
pub const MAX_BRAND_LEN: u64 = 200;
/// Longest pack size accepted, matching the `package_size` CHECK.
pub const MAX_PACKAGE_SIZE_LEN: u64 = 100;

/// `PUT /api/product-dislikes`: "I do not want this product again".
#[derive(Debug, Deserialize, Validate)]
pub struct ProductDislikeCreate {
    pub store: Store,
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: String,
    #[validate(length(min = 1, max = MAX_PRODUCT_NAME_LEN))]
    pub name: String,
    #[validate(length(max = MAX_BRAND_LEN))]
    pub brand: Option<String>,
    #[validate(length(max = MAX_PACKAGE_SIZE_LEN))]
    pub package_size: Option<String>,
}

/// One member's dislike, as the household sees it.
#[derive(Debug, Serialize)]
pub struct ProductDislikeResponse {
    pub store: Store,
    pub store_name: &'static str,
    pub product_id: String,
    pub name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub user_id: String,
    pub user_name: String,
    /// The dislike is the caller's own, so the caller may remove it.
    pub mine: bool,
    pub disliked_at: DateTime<Utc>,
}

impl ProductDislikeResponse {
    /// Shapes a row for the member `viewer_id`, who may or may not own it.
    pub fn for_viewer(row: ProductDislike, viewer_id: &str) -> Self {
        Self {
            store: row.store,
            store_name: row.store.display_name(),
            mine: row.user_id == viewer_id,
            product_id: row.product_id,
            name: row.product_name,
            brand: row.brand,
            package_size: row.package_size,
            user_id: row.user_id,
            user_name: row.user_name,
            disliked_at: row.disliked_at,
        }
    }
}

/// `PUT /api/grocery-items/{id}/dislike-override`: buy it this time anyway.
#[derive(Debug, Deserialize, Validate)]
pub struct DislikeOverrideCreate {
    pub store: Store,
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: String,
}

/// A dislike set aside for one list item.
#[derive(Debug, Serialize)]
pub struct DislikeOverrideResponse {
    pub grocery_item_id: Uuid,
    pub store: Store,
    pub product_id: String,
    pub overridden_by: String,
    pub overridden_at: DateTime<Utc>,
}

impl From<DislikeOverride> for DislikeOverrideResponse {
    fn from(row: DislikeOverride) -> Self {
        Self {
            grocery_item_id: row.grocery_item_id,
            store: row.store,
            product_id: row.product_id,
            overridden_by: row.overridden_by,
            overridden_at: row.overridden_at,
        }
    }
}
