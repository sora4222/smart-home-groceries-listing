//! `GET /api/access-logs` — the `/logs` page's data
//! (`docs/features/FEATURE_ACCESS_LOGS.md`).
//!
//! Needs a signed-in household member. There is no admin role: every member
//! may read the log.

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{AccessLogPageResponse, AccessLogQuery};
use crate::services::access_log;
use crate::state::AppState;

/// Routes under `/api/access-logs`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/access-logs", get(list_page))
}

/// `GET /api/access-logs?before=&limit=&hide_health_checks=` — one page of
/// answered requests, newest first. Health checks are hidden unless
/// `hide_health_checks=false`.
async fn list_page(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<AccessLogQuery>,
) -> Result<Json<AccessLogPageResponse>, ApiError> {
    let page = access_log::recent(
        &state.pool,
        query.before,
        query.limit,
        query.hide_health_checks.unwrap_or(true),
    )
    .await?;
    Ok(Json(page.into()))
}
