//! Bodies for trolley handoffs: the web app's side (create, read) and the
//! store tab's side (claim, report).
//!
//! The store tab is the "Fill trolley" bookmarklet running on the store's own
//! website. It authenticates with the store-tab secret **in the body**, not a
//! header, so its requests stay CORS "simple requests" with no preflight.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::db::{
    TrolleyHandoff, TrolleyHandoffLine, TrolleyHandoffStatus, TrolleyLineOutcome,
};
use crate::models::schemas::MAX_PRODUCT_ID_LEN;
use crate::services::stores::Store;
use crate::services::trolley_handoffs::store_tab::{ReportedLine, StoreTabLine};
use crate::services::trolley_handoffs::{ClaimedHandoff, HandoffWithLines};

/// Longest store-tab secret accepted in a body.
const MAX_SECRET_LEN: u64 = 200;

/// Longest problem text a report may carry, matching the column's CHECK.
const MAX_PROBLEM_LEN: u64 = 300;

/// Most products one report may describe.
const MAX_REPORT_LINES: u64 = 200;

/// `POST /api/trolley-handoffs`: which store's trolley to fill.
#[derive(Debug, Deserialize, Validate)]
pub struct TrolleyHandoffCreate {
    pub store: Store,
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
