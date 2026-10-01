//! Cross-origin requests from the web app, through the real router.
//!
//! The web app is served from its own origin, so the browser asks first
//! (a preflight `OPTIONS`) before every method other than a simple GET/POST.
//! A method missing here works in `curl` and in the integration tests but is
//! blocked in every browser.

mod common;

use axum::body::Body;
use axum::http::{Method, Request};
use common::TestApp;
use sqlx::PgPool;

/// The `Access-Control-Allow-Methods` a preflight for `method` is answered with.
async fn allowed_methods(app: &TestApp, method: &str) -> String {
    let (_, _, headers) = app
        .send_with_headers(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/api/grocery-items/00000000-0000-0000-0000-000000000000/selection")
                .header("origin", "http://localhost:3000")
                .header("access-control-request-method", method)
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    headers
        .get("access-control-allow-methods")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default()
}

#[sqlx::test]
async fn the_web_app_may_use_every_method_the_api_serves(pool: PgPool) {
    let app = TestApp::new(pool);

    for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
        let allowed = allowed_methods(&app, method).await;
        assert!(
            allowed.split(',').any(|m| m.trim() == method),
            "{method} is not allowed cross-origin: {allowed:?}"
        );
    }
}
