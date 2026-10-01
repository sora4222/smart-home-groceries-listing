//! Bodies for trolley handoffs: the web app's side (create, read) and the
//! store tab's side (claim, report).
//!
//! The store tab is the "Fill trolley" bookmarklet running on the store's own
//! website. It authenticates with the store-tab secret **in the body**, not a
//! header, so its requests stay CORS "simple requests" with no preflight.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::db::{
    DeliveryOutcome, DeliveryTimeOfDay, TrolleyHandoff, TrolleyHandoffLine, TrolleyHandoffStatus,
    TrolleyLineOutcome,
};
use crate::models::schemas::MAX_PRODUCT_ID_LEN;
use crate::services::stores::Store;
use crate::services::trolley_handoffs::delivery::{DeliveryRequest, ReportedDelivery};
use crate::services::trolley_handoffs::store_tab::{ReportedLine, StoreTabLine};
use crate::services::trolley_handoffs::{ClaimedHandoff, HandoffWithLines};

/// Longest store-tab secret accepted in a body.
const MAX_SECRET_LEN: u64 = 200;

/// Longest problem text a report may carry, matching the column's CHECK.
const MAX_PROBLEM_LEN: u64 = 300;

/// Most products one report may describe.
const MAX_REPORT_LINES: u64 = 200;

/// Longest delivery window label accepted, matching the column's CHECK.
const MAX_WINDOW_LABEL_LEN: u64 = 100;

/// `POST /api/trolley-handoffs`: which store's trolley to fill, and when the
/// delivery should come. Without `delivery`: the next day, any time.
#[derive(Debug, Deserialize, Validate)]
pub struct TrolleyHandoffCreate {
    pub store: Store,
    #[serde(default)]
    pub delivery: DeliveryWanted,
}

/// The delivery time wanted. Both fields are optional.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeliveryWanted {
    /// `None` = the day after the store's today.
    pub date: Option<NaiveDate>,
    #[serde(default)]
    pub time_of_day: DeliveryTimeOfDay,
}

impl From<&DeliveryWanted> for DeliveryRequest {
    fn from(wanted: &DeliveryWanted) -> Self {
        Self {
            date: wanted.date,
            time_of_day: wanted.time_of_day,
        }
    }
}

/// The delivery window as the web app reads it: what was asked for, and what
/// the store tab did (all `None` until it reports).
#[derive(Debug, Serialize)]
pub struct TrolleyHandoffDelivery {
    pub requested: DeliveryWanted,
    pub outcome: Option<DeliveryOutcome>,
    /// The store's own words, e.g. "7:00am - 10:00am".
    pub window_label: Option<String>,
    /// Store-local time, no offset.
    pub window_start: Option<NaiveDateTime>,
    pub window_end: Option<NaiveDateTime>,
    pub fee: Option<Decimal>,
    pub problem: Option<String>,
}

/// A handoff as the web app reads it.
#[derive(Debug, Serialize)]
pub struct TrolleyHandoffResponse {
    pub id: Uuid,
    pub store: Store,
    pub store_name: &'static str,
    pub status: TrolleyHandoffStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub reported_at: Option<DateTime<Utc>>,
    pub delivery: TrolleyHandoffDelivery,
    pub lines: Vec<TrolleyHandoffLineResponse>,
}

/// One list item's line in a handoff.
#[derive(Debug, Serialize)]
pub struct TrolleyHandoffLineResponse {
    pub grocery_item_id: Uuid,
    pub product_id: String,
    pub name: String,
    pub quantity: i32,
    pub outcome: Option<TrolleyLineOutcome>,
    pub problem: Option<String>,
}

impl From<HandoffWithLines> for TrolleyHandoffResponse {
    fn from(HandoffWithLines { handoff, lines }: HandoffWithLines) -> Self {
        let TrolleyHandoff {
            id,
            store,
            status,
            created_at,
            expires_at,
            claimed_at,
            reported_at,
            delivery_date,
            delivery_time_of_day,
            delivery_outcome,
            delivery_window_label,
            delivery_window_start,
            delivery_window_end,
            delivery_fee,
            delivery_problem,
            ..
        } = handoff;
        Self {
            id,
            store,
            store_name: store.display_name(),
            status,
            created_at,
            expires_at,
            claimed_at,
            reported_at,
            delivery: TrolleyHandoffDelivery {
                requested: DeliveryWanted {
                    date: delivery_date,
                    time_of_day: delivery_time_of_day,
                },
                outcome: delivery_outcome,
                window_label: delivery_window_label,
                window_start: delivery_window_start,
                window_end: delivery_window_end,
                fee: delivery_fee,
                problem: delivery_problem,
            },
            lines: lines.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TrolleyHandoffLine> for TrolleyHandoffLineResponse {
    fn from(line: TrolleyHandoffLine) -> Self {
        Self {
            grocery_item_id: line.grocery_item_id,
            product_id: line.product_id,
            name: line.product_name,
            quantity: line.quantity,
            outcome: line.outcome,
            problem: line.problem,
        }
    }
}

/// `GET /api/trolley-handoffs/store-tab-secret`: what the web app builds the
/// bookmarklet with.
#[derive(Debug, Serialize)]
pub struct StoreTabSecretResponse {
    pub secret: String,
}

/// `POST /api/store-tab/trolley-handoffs/claim`, sent by the bookmarklet.
#[derive(Debug, Deserialize, Validate)]
pub struct StoreTabClaimRequest {
    #[validate(length(max = MAX_SECRET_LEN))]
    pub secret: String,
    pub store: Store,
}

/// The handoff the store tab is to fill: one line per product.
#[derive(Debug, Serialize)]
pub struct StoreTabClaimResponse {
    pub handoff_id: Uuid,
    pub store: Store,
    /// The delivery window to reserve before adding products.
    pub delivery: DeliveryWanted,
    pub lines: Vec<StoreTabLineResponse>,
}

/// One product for the store tab: add `quantity` more of `product_id`.
#[derive(Debug, Serialize)]
pub struct StoreTabLineResponse {
    pub product_id: String,
    pub name: String,
    pub quantity: i32,
}

impl From<ClaimedHandoff> for StoreTabClaimResponse {
    fn from(claimed: ClaimedHandoff) -> Self {
        Self {
            handoff_id: claimed.handoff.id,
            store: claimed.handoff.store,
            delivery: DeliveryWanted {
                date: claimed.handoff.delivery_date,
                time_of_day: claimed.handoff.delivery_time_of_day,
            },
            lines: claimed.lines.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<StoreTabLine> for StoreTabLineResponse {
    fn from(line: StoreTabLine) -> Self {
        Self {
            product_id: line.product_id,
            name: line.product_name,
            quantity: line.quantity,
        }
    }
}

/// `POST /api/store-tab/trolley-handoffs/{id}/report`, sent by the
/// bookmarklet once every product has been tried.
#[derive(Debug, Deserialize, Validate)]
pub struct StoreTabReportRequest {
    #[validate(length(max = MAX_SECRET_LEN))]
    pub secret: String,
    #[validate(length(min = 1, max = MAX_REPORT_LINES), nested)]
    pub lines: Vec<StoreTabReportedLine>,
    /// What happened to the delivery window; absent from older bookmarklets.
    #[validate(nested)]
    pub delivery: Option<StoreTabReportedDelivery>,
}

/// The delivery window the store tab reserved, kept, or could not reserve.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StoreTabReportedDelivery {
    pub outcome: DeliveryOutcome,
    #[validate(length(min = 1, max = MAX_WINDOW_LABEL_LEN))]
    pub window_label: Option<String>,
    pub window_start: Option<NaiveDateTime>,
    pub window_end: Option<NaiveDateTime>,
    #[validate(custom(function = "not_negative"))]
    pub fee: Option<Decimal>,
    #[validate(length(max = MAX_PROBLEM_LEN))]
    pub problem: Option<String>,
}

impl From<StoreTabReportedDelivery> for ReportedDelivery {
    fn from(d: StoreTabReportedDelivery) -> Self {
        Self {
            outcome: d.outcome,
            window_label: d.window_label,
            window_start: d.window_start,
            window_end: d.window_end,
            fee: d.fee,
            problem: d.problem,
        }
    }
}

/// A delivery fee is never negative.
fn not_negative(fee: &Decimal) -> Result<(), validator::ValidationError> {
    if fee.is_sign_negative() {
        Err(validator::ValidationError::new("negative_fee"))
    } else {
        Ok(())
    }
}

/// What happened to one product in the store's trolley. (`Serialize` is
/// needed by `validator`'s nested error reporting.)
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StoreTabReportedLine {
    #[validate(length(min = 1, max = MAX_PRODUCT_ID_LEN))]
    pub product_id: String,
    pub outcome: TrolleyLineOutcome,
    #[validate(length(max = MAX_PROBLEM_LEN))]
    pub problem: Option<String>,
}

impl From<StoreTabReportedLine> for ReportedLine {
    fn from(line: StoreTabReportedLine) -> Self {
        Self {
            product_id: line.product_id,
            outcome: line.outcome,
            problem: line.problem,
        }
    }
}
