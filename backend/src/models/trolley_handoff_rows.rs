//! Row types for `trolley_handoffs` and `trolley_handoff_lines`, and the
//! enumerations their TEXT + CHECK columns hold (migrations `0005`, `0006`).

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::stores::Store;

/// Where a trolley handoff is in its life (`trolley_handoffs.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TrolleyHandoffStatus {
    /// Created by the web app; the store tab has not claimed it yet.
    WaitingForStoreTab,
    /// The bookmarklet in the store's tab took it and is filling the trolley.
    ClaimedByStoreTab,
    /// Every line went into the store's trolley.
    Filled,
    /// The report came back with at least one line the store refused.
    FilledWithProblems,
    /// A newer handoff for the same store was created before this was claimed.
    Replaced,
}

/// Chosen products waiting for the household's own browser tab to put them in
/// a store's online trolley (`trolley_handoffs`).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrolleyHandoff {
    pub id: Uuid,
    pub store: Store,
    pub status: TrolleyHandoffStatus,
    /// The household member who pressed "Send to the store".
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    /// After this a waiting handoff can no longer be claimed.
    pub expires_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub reported_at: Option<DateTime<Utc>>,
    /// The delivery day asked for; `None` = the day after the store's today.
    pub delivery_date: Option<NaiveDate>,
    pub delivery_time_of_day: DeliveryTimeOfDay,
    /// What the store tab did about the delivery window, once it reported.
    pub delivery_outcome: Option<DeliveryOutcome>,
    /// The store's own words for the window, e.g. "7:00am - 10:00am".
    pub delivery_window_label: Option<String>,
    /// The window in the store's local time.
    pub delivery_window_start: Option<NaiveDateTime>,
    pub delivery_window_end: Option<NaiveDateTime>,
    pub delivery_fee: Option<Decimal>,
    /// Why no window was reserved, or why another day was used.
    pub delivery_problem: Option<String>,
}

/// What the store tab reported for one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TrolleyLineOutcome {
    Added,
    Failed,
}

/// One list item's product and quantity inside a handoff
/// (`trolley_handoff_lines`).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrolleyHandoffLine {
    pub handoff_id: Uuid,
    pub grocery_item_id: Uuid,
    /// The store's own id for the product (a Woolworths stockcode).
    pub product_id: String,
    pub product_name: String,
    /// How many to put in the trolley: the list item's quantity.
    pub quantity: i32,
    /// `None` until the store tab reports back.
    pub outcome: Option<TrolleyLineOutcome>,
    /// Why the store refused the line, in the store's words when it gave any.
    pub problem: Option<String>,
}

/// The part of the day a delivery window should start in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DeliveryTimeOfDay {
    /// Whatever window is cheapest, then earliest.
    #[default]
    Any,
    /// Starts before 12pm.
    Morning,
    /// Starts from 12pm, before 5pm.
    Afternoon,
    /// Starts at 5pm or later.
    Evening,
}

/// What the store tab did about the delivery window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DeliveryOutcome {
    /// A window was chosen and reserved on the store's website.
    Reserved,
    /// The window already reserved on the account fitted the request.
    Kept,
    /// No window could be reserved; `delivery_problem` says why.
    Failed,
}
