//! Integration tests for the intake confirmation queue's lifecycle.
//!
//! These are the previous `tests/integration/test_voice_requests.py` suite,
//! case for case, plus the cases the Python version could not express (the
//! 404/409 split, the validation limits). Duplicate handling and merging
//! live in `voice_duplicates.rs`.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// A name unique to this test, so tests never collide on duplicate detection.
fn unique_item() -> String {
    format!("test-item-{}", &Uuid::new_v4().simple().to_string()[..8])
}

#[sqlx::test]
async fn webhook_requires_shared_secret(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app
        .post_webhook_unauthenticated(&json!({ "item": unique_item(), "quantity": 1 }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn webhook_rejects_a_wrong_secret(pool: PgPool) {
    let app = TestApp::new(pool);
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/api/voice-requests")
        .header("content-type", "application/json")
        .header("x-webhook-secret", "not-the-secret")
        .body(axum::body::Body::from(
            json!({ "item": "milk", "quantity": 1 }).to_string(),
        ))
        .unwrap();
    let (status, _) = app.send(request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn webhook_creates_pending_request(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();

    let (status, body) = app
        .post_webhook(&json!({ "item": item, "quantity": 2 }))
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["parsed_name"], item.as_str());
    assert_eq!(body["parsed_quantity"], 2);
    assert_eq!(body["status"], "pending");
    assert_eq!(body["source"], "webhook");
}

#[sqlx::test]
async fn webhook_defaults_quantity_to_one(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, body) = app.post_webhook(&json!({ "item": unique_item() })).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["parsed_quantity"], 1);
}

#[sqlx::test]
async fn pending_list_shows_unconfirmed_items(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    app.create_pending(&item, 1).await;

    let (status, body) = app.get("/api/voice-requests").await;

    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["parsed_name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&item.as_str()));
}

#[sqlx::test]
async fn accept_moves_item_to_active_grocery_list(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let request_id = app.create_pending(&item, 3).await;

    let (status, body) = app
        .post(&format!("/api/voice-requests/{request_id}/accept"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["voice_request"]["status"], "accepted");
    assert_eq!(body["grocery_item"]["name"], item.as_str());
    assert_eq!(body["grocery_item"]["quantity"], 3);
    assert_eq!(body["grocery_item"]["status"], "active");
    assert_eq!(body["grocery_item"]["source"], "voice");

    let (_, listed) = app.get("/api/grocery-items").await;
    assert!(listed
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["name"] == item.as_str()));
}

#[sqlx::test]
async fn accept_applies_user_corrections(pool: PgPool) {
    let app = TestApp::new(pool);
    let corrected = unique_item();
    let request_id = app.create_pending("mispelled itme", 1).await;

    let (status, body) = app
        .post_json(
            &format!("/api/voice-requests/{request_id}/accept"),
            &json!({ "name": corrected, "quantity": 5 }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["grocery_item"]["name"], corrected.as_str());
    assert_eq!(body["grocery_item"]["quantity"], 5);
}

#[sqlx::test]
async fn accept_with_an_empty_body_keeps_what_was_heard(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let request_id = app.create_pending(&item, 2).await;

    let (status, body) = app
        .post_json(
            &format!("/api/voice-requests/{request_id}/accept"),
            &json!({}),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["grocery_item"]["name"], item.as_str());
    assert_eq!(body["grocery_item"]["quantity"], 2);
}

#[sqlx::test]
async fn reject_discards_the_request(pool: PgPool) {
    let app = TestApp::new(pool);
    let request_id = app.create_pending(&unique_item(), 1).await;

    let (status, body) = app
        .post(&format!("/api/voice-requests/{request_id}/reject"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "rejected");

    let (_, pending) = app.get("/api/voice-requests").await;
    assert!(!pending
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == request_id.as_str()));
}

#[sqlx::test]
async fn accepting_a_rejected_request_undoes_the_rejection(pool: PgPool) {
    // Supports the "dulled, undo-on-hover" reject UX in the web app.
    let app = TestApp::new(pool);
    let request_id = app.create_pending(&unique_item(), 1).await;
    app.post(&format!("/api/voice-requests/{request_id}/reject"))
        .await;

    let (status, body) = app
        .post(&format!("/api/voice-requests/{request_id}/accept"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["voice_request"]["status"], "accepted");
}

#[sqlx::test]
async fn cannot_accept_an_already_accepted_request(pool: PgPool) {
    let app = TestApp::new(pool);
    let request_id = app.create_pending(&unique_item(), 1).await;
    app.post(&format!("/api/voice-requests/{request_id}/accept"))
        .await;

    let (status, _) = app
        .post(&format!("/api/voice-requests/{request_id}/accept"))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn cannot_reject_an_already_decided_request(pool: PgPool) {
    let app = TestApp::new(pool);
    let request_id = app.create_pending(&unique_item(), 1).await;
    app.post(&format!("/api/voice-requests/{request_id}/reject"))
        .await;

    let (status, _) = app
        .post(&format!("/api/voice-requests/{request_id}/reject"))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn an_unknown_request_id_is_a_404_not_a_409(pool: PgPool) {
    let app = TestApp::new(pool);
    let missing = Uuid::new_v4();

    let (accept_status, _) = app
        .post(&format!("/api/voice-requests/{missing}/accept"))
        .await;
    let (reject_status, _) = app
        .post(&format!("/api/voice-requests/{missing}/reject"))
        .await;

    assert_eq!(accept_status, StatusCode::NOT_FOUND);
    assert_eq!(reject_status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_blank_name_correction_is_ignored_rather_than_stored(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let request_id = app.create_pending(&item, 1).await;

    let (status, body) = app
        .post_json(
            &format!("/api/voice-requests/{request_id}/accept"),
            &json!({ "name": "   " }),
        )
        .await;

    // A user who clears the field has not asked for a blank name — the item
    // keeps what the intake channel heard rather than erroring or being lost.
    assert_eq!(status, StatusCode::OK, "got: {body}");
    assert_eq!(body["grocery_item"]["name"], item.as_str());
}

#[sqlx::test]
async fn the_webhook_validates_its_payload(pool: PgPool) {
    let app = TestApp::new(pool);

    for body in [
        json!({ "item": "", "quantity": 1 }),
        json!({ "item": "milk", "quantity": 0 }),
        json!({ "item": "milk", "quantity": 1000 }),
        json!({ "item": "x".repeat(201), "quantity": 1 }),
    ] {
        let (status, _) = app.post_webhook(&body).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "should have rejected {body}"
        );
    }
}

#[sqlx::test]
async fn health_needs_no_authentication(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, body) = app.get("/api/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}
