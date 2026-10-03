//! The middleware that logs every request (`docs/features/FEATURE_ACCESS_LOGS.md`).
//!
//! It runs outermost, so requests refused by the timeout, the body limit or
//! CORS are logged too. The signed-in user comes from the route's own auth
//! check through [`RequestUser`], so no token is verified twice.

mod request_info;

use std::time::Instant;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::request_user::RequestUser;
use crate::services::access_log::{self, user_label, AccessLogEntry};
use crate::state::AppState;

pub use request_info::{CF_CONNECTING_IP, X_FORWARDED_FOR};

/// Logs one request once its response is ready, then hands the response on.
pub async fn record_access(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let started = Instant::now();
    let method = request.method().to_string();
    let path = request_info::loggable_path(request.uri().path());
    let source_ip = request_info::source_ip(request.extensions());
    let forwarded_for = request_info::forwarded_for(request.headers());

    let user = RequestUser::default();
    request.extensions_mut().insert(user.clone());

    let response = next.run(request).await;

    access_log::record(
        &state.pool,
        AccessLogEntry {
            source_ip,
            forwarded_for,
            user_id: user_label(user.id()),
            method,
            path,
            status_code: response.status().as_u16(),
            duration_ms: u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX),
        },
    );
    response
}
