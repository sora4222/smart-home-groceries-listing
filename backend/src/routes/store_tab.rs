//! The store tab's side of trolley handoffs: the "Fill trolley" bookmarklet,
//! running on the store's own website in the household's logged-in tab,
//! claims a handoff and reports what it put in the trolley.
//!
//! These requests come from the **store's** origin, not the web app's:
//!
//! * They authenticate with `STORE_TAB_SECRET` in the JSON body and send it
//!   as `text/plain`, so the browser makes a CORS simple request with no
//!   preflight ([`AnyContentTypeJson`]).
//! * [`allow_store_origins`] adds `Access-Control-Allow-Origin` for the
//!   stores' websites on these routes only, so the bookmarklet can read the
//!   answers. The rest of the API stays closed to those origins.

use axum::extract::{Path, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::secret::verify_shared_secret;
use crate::error::ApiError;
use crate::models::schemas::{
    StoreTabClaimRequest, StoreTabClaimResponse, StoreTabReportRequest, TrolleyHandoffResponse,
};
use crate::routes::extract::AnyContentTypeJson;
use crate::services::trolley_handoffs::delivery::ReportedDelivery;
use crate::services::trolley_handoffs::store_tab::ReportedLine;
use crate::services::trolley_handoffs::TrolleyHandoffService;
use crate::state::AppState;

/// `/api/store-tab/trolley-handoffs/*`, authenticated by `STORE_TAB_SECRET`.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/store-tab/trolley-handoffs/claim", post(claim_handoff))
        .route(
            "/api/store-tab/trolley-handoffs/{handoff_id}/report",
            post(report_handoff),
        )
        .layer(middleware::from_fn_with_state(state, allow_store_origins))
}

/// `POST /api/store-tab/trolley-handoffs/claim` — 200 the newest waiting
/// handoff for the store, now claimed; 204 when nothing is waiting.
async fn claim_handoff(
    State(state): State<AppState>,
    AnyContentTypeJson(body): AnyContentTypeJson<StoreTabClaimRequest>,
) -> Result<Response, ApiError> {
    check_secret(&state, &body.secret)?;
    let claimed = TrolleyHandoffService::new(&state.pool)
        .claim_for_store_tab(body.store)
        .await?;
    Ok(match claimed {
        Some(claimed) => Json(StoreTabClaimResponse::from(claimed)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    })
}

/// `POST /api/store-tab/trolley-handoffs/{id}/report` — record what went
/// into the trolley and which delivery window was reserved. 404 unknown, 409
/// not claimed, 422 incomplete report or a delivery window that makes no sense.
async fn report_handoff(
    State(state): State<AppState>,
    Path(handoff_id): Path<Uuid>,
    AnyContentTypeJson(body): AnyContentTypeJson<StoreTabReportRequest>,
) -> Result<Json<TrolleyHandoffResponse>, ApiError> {
    check_secret(&state, &body.secret)?;
    let report: Vec<ReportedLine> = body.lines.into_iter().map(Into::into).collect();
    let delivery: Option<ReportedDelivery> = body.delivery.map(Into::into);
    let closed = TrolleyHandoffService::new(&state.pool)
        .record_store_tab_report(handoff_id, &report, delivery.as_ref())
        .await?;
    Ok(Json(closed.into()))
}

/// 401 unless the body carries the configured store-tab secret.
fn check_secret(state: &AppState, presented: &str) -> Result<(), ApiError> {
    verify_shared_secret(
        &state.settings.store_tab_secret,
        Some(presented),
        "STORE_TAB_SECRET",
    )
}

/// Lets the stores' websites read these routes' answers, errors included.
async fn allow_store_origins(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    let mut response = next.run(request).await;
    if let Some(origin) = origin.filter(|o| is_store_website(&state, o)) {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    }
    response
}

/// The origin is one of the stores' configured websites.
fn is_store_website(state: &AppState, origin: &HeaderValue) -> bool {
    let stores = &state.settings.stores;
    [&stores.woolworths_base_url, &stores.coles_base_url]
        .into_iter()
        .any(|base| origin.as_bytes() == base.trim_end_matches('/').as_bytes())
}
