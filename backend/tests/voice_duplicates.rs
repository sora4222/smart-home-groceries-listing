//! Integration tests for duplicate handling on the confirmation queue.
//!
//! The spec's rule is that accepting an item already on the list must ask the
//! user "add another or update the existing quantity?" rather than silently
//! creating a second row. These cover that, the `?merge=true` path, and the
//! normalisation that decides what counts as the same item.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use sqlx::PgPool;
use uuid::Uuid;

/// A name unique to this test, so tests never collide on duplicate detection.
fn unique_item() -> String {
    format!("test-item-{}", &Uuid::new_v4().simple().to_string()[..8])
}

#[sqlx::test]
async fn accepting_a_duplicate_active_item_returns_409_with_existing_item(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 1).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;

    // Same item, different case: duplicate detection is case-insensitive.
    let second = app.create_pending(&item.to_uppercase(), 2).await;
    let (status, body) = app
        .post(&format!("/api/voice-requests/{second}/accept"))
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    let message = body["detail"]["message"].as_str().unwrap();
    assert!(message.contains("already on the list"), "got: {message}");
    assert_eq!(body["detail"]["existing_item"]["name"], item.as_str());
    assert_eq!(body["detail"]["existing_item"]["quantity"], 1);
}

#[sqlx::test]
async fn a_duplicate_conflict_leaves_the_request_still_pending(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 1).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;

    let second = app.create_pending(&item, 2).await;
    app.post(&format!("/api/voice-requests/{second}/accept"))
        .await;

    // The rolled-back accept must not have consumed the request: the user
    // still has to choose, so the card stays in the queue.
    let (_, pending) = app.get("/api/voice-requests").await;
    assert!(pending
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == second.as_str()));
}

#[sqlx::test]
async fn merge_flag_increments_existing_quantity_instead_of_duplicating(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 1).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;

    let second = app.create_pending(&item, 2).await;
    let (status, body) = app
        .post(&format!("/api/voice-requests/{second}/accept?merge=true"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["grocery_item"]["quantity"], 3);

    let (_, listed) = app.get("/api/grocery-items").await;
    let matching: Vec<_> = listed
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["name"] == item.as_str())
        .collect();
    assert_eq!(matching.len(), 1, "merging must not create a second row");
}

#[sqlx::test]
async fn merging_is_capped_at_the_maximum_quantity(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 999).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;

    let second = app.create_pending(&item, 5).await;
    let (status, body) = app
        .post(&format!("/api/voice-requests/{second}/accept?merge=true"))
        .await;

    // Without the cap this would violate the column's CHECK constraint and
    // surface as a 500.
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["grocery_item"]["quantity"], 999);
}

#[sqlx::test]
async fn whitespace_only_differences_are_treated_as_duplicates(pool: PgPool) {
    let app = TestApp::new(pool);
    let first = app.create_pending("full cream milk", 1).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;

    let second = app.create_pending("  Full   Cream   Milk  ", 1).await;
    let (status, _) = app
        .post(&format!("/api/voice-requests/{second}/accept"))
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
}
