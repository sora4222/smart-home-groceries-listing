//! Integration tests for Alexa intake, forwarded by the bridge sidecar.
//!
//! The sidecar verifies Alexa's own request signature; what this endpoint
//! must get right is authenticating the sidecar and de-duplicating retries.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

fn unique_external_id() -> String {
    format!("amzn1.echo-api.request.{}", Uuid::new_v4())
}

#[sqlx::test]
async fn records_a_spoken_item_as_pending(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app
        .post_alexa(&json!({
            "item": "oat milk",
            "quantity": 2,
            "external_id": unique_external_id(),
            "raw_text": "add two oat milk to my shopping list",
        }))
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["source"], "alexa");
    assert_eq!(body["status"], "pending");
    assert_eq!(body["parsed_name"], "oat milk");
    assert_eq!(body["parsed_quantity"], 2);
    assert_eq!(body["raw_text"], "add two oat milk to my shopping list");
}

#[sqlx::test]
async fn an_alexa_item_needs_manual_acceptance_like_any_other(pool: PgPool) {
    let app = TestApp::new(pool);
    app.post_alexa(&json!({ "item": "rice", "external_id": unique_external_id() }))
        .await;

    // Nothing reaches the active list without a household member accepting.
    let (_, listed) = app.get("/api/grocery-items").await;
    assert!(listed.as_array().unwrap().is_empty());

    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn requires_the_bridge_secret(pool: PgPool) {
    let app = TestApp::new(pool);
    let body = json!({ "item": "rice", "external_id": unique_external_id() });

    let (missing, _) = app.post_alexa_with_secret(&body, None).await;
    let (wrong, _) = app
        .post_alexa_with_secret(&body, Some("not-the-secret"))
        .await;
    let (prefix, _) = app.post_alexa_with_secret(&body, Some("test-bridge")).await;

    assert_eq!(missing, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong, StatusCode::UNAUTHORIZED);
    assert_eq!(prefix, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn the_webhook_secret_does_not_open_the_alexa_endpoint(pool: PgPool) {
    // The two secrets are deliberately separate, so leaking the webhook's
    // does not also grant the Alexa path.
    let app = TestApp::new(pool);
    let (status, _) = app
        .post_alexa_with_secret(
            &json!({ "item": "rice", "external_id": unique_external_id() }),
            Some(common::TEST_WEBHOOK_SECRET),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_retried_delivery_does_not_create_a_second_card(pool: PgPool) {
    // Alexa retries an endpoint it believes timed out; that must be a no-op.
    let app = TestApp::new(pool);
    let external_id = unique_external_id();
    let body = json!({ "item": "butter", "quantity": 1, "external_id": external_id });

    let (first_status, first) = app.post_alexa(&body).await;
    let (retry_status, retry) = app.post_alexa(&body).await;

    assert_eq!(first_status, StatusCode::CREATED);
    assert_eq!(retry_status, StatusCode::OK, "a retry is not a new item");
    assert_eq!(first["id"], retry["id"]);

    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn two_utterances_of_the_same_item_are_both_recorded(pool: PgPool) {
    // Distinct external ids mean the user really did ask twice.
    let app = TestApp::new(pool);

    app.post_alexa(&json!({ "item": "butter", "external_id": unique_external_id() }))
        .await;
    app.post_alexa(&json!({ "item": "butter", "external_id": unique_external_id() }))
        .await;

    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending.as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn an_item_without_an_external_id_is_still_accepted(pool: PgPool) {
    // Some skill shapes cannot supply one; the item must not be lost.
    let app = TestApp::new(pool);

    let (status, body) = app.post_alexa(&json!({ "item": "bread" })).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["parsed_quantity"], 1);
    assert_eq!(body["raw_text"], "bread");
}

#[sqlx::test]
async fn an_accepted_alexa_item_reaches_the_grocery_list(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, created) = app
        .post_alexa(&json!({
            "item": "tinned tomatoes",
            "quantity": 4,
            "external_id": unique_external_id(),
        }))
        .await;
    let request_id = created["id"].as_str().unwrap();

    let (status, body) = app
        .post(&format!("/api/voice-requests/{request_id}/accept"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["grocery_item"]["name"], "tinned tomatoes");
    assert_eq!(body["grocery_item"]["quantity"], 4);
    assert_eq!(body["grocery_item"]["source"], "voice");
}

#[sqlx::test]
async fn validates_its_payload(pool: PgPool) {
    let app = TestApp::new(pool);

    for body in [
        json!({ "quantity": 1 }),
        json!({ "item": "", "quantity": 1 }),
        json!({ "item": "milk", "quantity": -1 }),
        json!({ "item": "milk", "quantity": 1000 }),
        json!({ "item": "x".repeat(201) }),
        json!({ "item": "milk", "external_id": "x".repeat(256) }),
    ] {
        let (status, _) = app.post_alexa(&body).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "should have rejected {body}"
        );
    }
}
