//! Every household route refuses a request with no session (spec:
//! "Unauthenticated requests to all API routes except the Alexa webhook
//! return 401").
//!
//! **Adding a route? Add it to [`household_routes`].** The intake endpoints
//! and the store tab use shared secrets instead and are listed separately, so
//! a route that is in neither list is easy to spot in review.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::TestApp;
use sqlx::PgPool;

/// A path whose id segments need any well-formed UUID.
const ID: &str = "00000000-0000-0000-0000-000000000001";

/// Routes that need a signed-in household member.
fn household_routes() -> Vec<(Method, String)> {
    let item = format!("/api/grocery-items/{ID}");
    vec![
        (Method::GET, "/api/voice-requests".into()),
        (Method::POST, format!("/api/voice-requests/{ID}/accept")),
        (Method::POST, format!("/api/voice-requests/{ID}/reject")),
        (Method::GET, "/api/grocery-items".into()),
        (Method::POST, "/api/grocery-items".into()),
        (Method::POST, "/api/grocery-items/commit".into()),
        (Method::POST, "/api/grocery-items/release".into()),
        (Method::PATCH, item.clone()),
        (Method::DELETE, item.clone()),
        (Method::GET, format!("{item}/products")),
        (Method::PUT, format!("{item}/selection")),
        (Method::DELETE, format!("{item}/selection")),
        (Method::GET, "/api/item-selections".into()),
        (Method::GET, "/api/item-rules".into()),
        (Method::POST, "/api/item-rules".into()),
        (Method::PATCH, format!("/api/item-rules/{ID}")),
        (Method::DELETE, format!("/api/item-rules/{ID}")),
        (Method::GET, "/api/order-review".into()),
        (Method::GET, "/api/triage?tab=held".into()),
        (Method::POST, format!("/api/triage/{ID}/accept")),
        (Method::POST, format!("/api/triage/{ID}/reject")),
        (Method::POST, "/api/trolley-handoffs".into()),
        (Method::GET, "/api/trolley-handoffs/store-tab-secret".into()),
        (Method::GET, format!("/api/trolley-handoffs/{ID}")),
    ]
}

/// A request with a JSON body, so a body extractor cannot answer first.
fn request(method: Method, uri: &str, authorization: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(value) = authorization {
        builder = builder.header("authorization", value);
    }
    builder.body(Body::from("{}")).unwrap()
}

#[sqlx::test]
async fn every_household_route_refuses_a_request_with_no_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    for (method, uri) in household_routes() {
        let (status, _) = app.send(request(method.clone(), &uri, None)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
    }
}

#[sqlx::test]
async fn every_household_route_refuses_a_token_that_does_not_verify(pool: PgPool) {
    let (_jwks, jwks_url) = common::clerk::serve_jwks().await;
    let app = TestApp::with_clerk(pool, &jwks_url);
    let forged = common::clerk::sign_forged(&common::clerk::session_claims("user_x"));
    let header = format!("Bearer {forged}");

    for (method, uri) in household_routes() {
        let (status, _) = app.send(request(method.clone(), &uri, Some(&header))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
    }
}

#[sqlx::test]
async fn health_needs_no_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let (status, _) = app.send(request(Method::GET, "/api/health", None)).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn intake_endpoints_take_their_shared_secret_not_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    // A missing secret is still a 401, but from the shared-secret check: a
    // valid secret with no session must get through.
    let (status, _) = app
        .post_alexa(&serde_json::json!({ "item": "milk", "quantity": 1 }))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = app
        .post_webhook(&serde_json::json!({ "item": "eggs", "quantity": 1 }))
        .await;
    assert_eq!(status, StatusCode::CREATED);
}
