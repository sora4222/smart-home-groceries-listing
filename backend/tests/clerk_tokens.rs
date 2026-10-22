//! Session tokens are verified end to end: signed like Clerk signs them,
//! checked against a JWKS endpoint, through the real router.
//!
//! `common::clerk` stands in for Clerk with test-only keys, so this needs no
//! Clerk account and makes no call outside the machine.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::clerk::{
    serve_jwks, session_claims, sign, sign_forged, sign_hs256_confusion, sign_unknown_key_id,
};
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

/// `GET /api/grocery-items` carrying `token` as a bearer token.
fn list_items_with(token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri("/api/grocery-items")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

/// The app wired to a fresh mock JWKS. The server is returned so it lives as
/// long as the test.
async fn app_with_clerk(pool: PgPool) -> (TestApp, wiremock::MockServer) {
    let (server, url) = serve_jwks().await;
    (TestApp::with_clerk(pool, &url), server)
}

#[sqlx::test]
async fn a_valid_session_token_is_let_in(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let token = sign(&session_claims("user_jesse"));

    let (status, body) = app.send(list_items_with(&token)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
}

#[sqlx::test]
async fn an_expired_token_is_refused(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let now = chrono::Utc::now().timestamp();
    // Well past jsonwebtoken's default 60-second leeway.
    let token = sign(&json!({ "sub": "user_jesse", "iat": now - 600, "exp": now - 300 }));

    let (status, _) = app.send(list_items_with(&token)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_token_signed_by_another_key_is_refused(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let token = sign_forged(&session_claims("user_jesse"));

    let (status, _) = app.send(list_items_with(&token)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_token_with_an_unknown_key_id_is_refused(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let token = sign_unknown_key_id(&session_claims("user_jesse"));

    let (status, _) = app.send(list_items_with(&token)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn an_hs256_token_cannot_borrow_the_public_key(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let token = sign_hs256_confusion(&session_claims("user_jesse"));

    let (status, _) = app.send(list_items_with(&token)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_token_with_no_subject_is_refused(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let now = chrono::Utc::now().timestamp();
    let token = sign(&json!({ "iat": now, "exp": now + 60 }));

    let (status, _) = app.send(list_items_with(&token)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn the_websocket_refuses_a_browser_with_no_session(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let address = app.serve().await;

    let refused = tokio_tungstenite::connect_async(format!("ws://{address}/ws")).await;

    match refused {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        other => panic!("expected a 401 handshake, got {other:?}"),
    }
}

#[sqlx::test]
async fn the_websocket_refuses_a_token_that_does_not_verify(pool: PgPool) {
    let (app, _jwks) = app_with_clerk(pool).await;
    let address = app.serve().await;
    let token = sign_forged(&session_claims("user_jesse"));

    let refused =
        tokio_tungstenite::connect_async(format!("ws://{address}/ws?token={token}")).await;

    assert!(refused.is_err(), "a forged token must not open a socket");
}

#[sqlx::test]
async fn the_websocket_lets_in_a_session_token_in_the_query(pool: PgPool) {
    // Browsers cannot set headers on a WebSocket, so the web app sends the
    // token as `?token=`.
    let (app, _jwks) = app_with_clerk(pool).await;
    let address = app.serve().await;
    let token = sign(&session_claims("user_jesse"));

    let opened = tokio_tungstenite::connect_async(format!("ws://{address}/ws?token={token}")).await;

    assert!(opened.is_ok(), "{:?}", opened.err());
}
