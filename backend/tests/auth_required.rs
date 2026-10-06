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
use grocery_backend::routes::alexa::BRIDGE_SECRET_HEADER;
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
        (Method::GET, "/api/stores/coles/products/123".into()),
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
        (Method::GET, "/api/purchase-orders".into()),
        (Method::DELETE, format!("/api/purchase-orders/{ID}")),
        (Method::POST, "/api/purchase-history/products".into()),
        (Method::POST, "/api/purchase-history/recategorise".into()),
        (Method::GET, "/api/spending".into()),
        (Method::GET, "/api/spending/item-prices?name=milk".into()),
        (Method::GET, "/api/access-logs".into()),
        (Method::GET, "/api/product-dislikes".into()),
        (Method::PUT, "/api/product-dislikes".into()),
        (
            Method::DELETE,
            "/api/product-dislikes/woolworths/123".into(),
        ),
        (Method::GET, "/api/dislike-overrides".into()),
        (Method::PUT, format!("{item}/dislike-override")),
        (
            Method::DELETE,
            format!("{item}/dislike-override/woolworths/123"),
        ),
        (Method::POST, format!("/api/triage/{ID}/restore")),
        (Method::GET, "/api/intake/settings".into()),
        (Method::PUT, "/api/intake/google-tasks".into()),
        (Method::POST, "/api/intake/google-tasks/sign-in".into()),
        (Method::DELETE, "/api/intake/google-tasks/sign-in".into()),
        (
            Method::POST,
            "/api/intake/google-tasks/sign-in/finish".into(),
        ),
        (Method::GET, "/api/intake/google-tasks/lists".into()),
        (Method::POST, "/api/intake/google-tasks/poll".into()),
        (Method::POST, "/api/dev/google-tasks/tasks".into()),
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

#[sqlx::test]
async fn alexa_remove_and_undo_take_the_bridge_secret_not_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    for uri in ["/api/intake/alexa/remove", "/api/intake/alexa/undo"] {
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .header(BRIDGE_SECRET_HEADER, common::TEST_BRIDGE_SECRET)
            .body(Body::from(r#"{"item":"milk"}"#))
            .unwrap();
        let (status, body) = app.send(request).await;
        // Nothing is on the list, so remove says 404 and undo has nothing to
        // undo; what matters is that no session was asked for.
        assert_ne!(status, StatusCode::UNAUTHORIZED, "{uri}: {body}");
    }
}
