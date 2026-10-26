//! Integration tests for undoing an accepted intake request: the Undo on the
//! Pending Requests toast.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// A name unique to this test, so tests never collide on duplicate detection.
fn unique_item() -> String {
    format!("undo-item-{}", &Uuid::new_v4().simple().to_string()[..8])
}

/// Every list item called `name`.
async fn items_named(app: &TestApp, name: &str) -> Vec<Value> {
    let (_, listed) = app.get("/api/grocery-items").await;
    listed
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["name"] == name)
        .cloned()
        .collect()
}

#[sqlx::test]
async fn undoing_a_new_entry_removes_it_and_requeues_the_request(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let id = app.create_pending(&item, 2).await;
    app.post(&format!("/api/voice-requests/{id}/accept")).await;

    let (status, body) = app
        .post(&format!("/api/voice-requests/{id}/undo-accept"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "pending");
    assert!(items_named(&app, &item).await.is_empty());
    let (_, pending) = app.get("/api/voice-requests").await;
    assert!(pending
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == id.as_str()));
}

#[sqlx::test]
async fn undoing_a_merge_takes_back_only_what_it_added(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 1).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;
    let second = app.create_pending(&item, 3).await;
    app.post(&format!(
        "/api/voice-requests/{second}/accept?on_duplicate=merge"
    ))
    .await;

    let (status, _) = app
        .post(&format!("/api/voice-requests/{second}/undo-accept"))
        .await;

    assert_eq!(status, StatusCode::OK);
    let items = items_named(&app, &item).await;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["quantity"], 1);
}

#[sqlx::test]
async fn undoing_a_capped_merge_takes_back_only_the_capped_amount(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 997).await;
    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;
    let second = app.create_pending(&item, 5).await;
    app.post(&format!(
        "/api/voice-requests/{second}/accept?on_duplicate=merge"
    ))
    .await;

    app.post(&format!("/api/voice-requests/{second}/undo-accept"))
        .await;

    assert_eq!(items_named(&app, &item).await[0]["quantity"], 997);
}

#[sqlx::test]
async fn undoing_a_request_that_was_not_accepted_is_a_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.create_pending(&unique_item(), 1).await;

    let (status, _) = app
        .post(&format!("/api/voice-requests/{id}/undo-accept"))
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn undoing_an_unknown_request_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app
        .post(&format!(
            "/api/voice-requests/{}/undo-accept",
            Uuid::new_v4()
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn undoing_after_the_item_was_removed_still_requeues(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let id = app.create_pending(&item, 1).await;
    let (_, accepted) = app.post(&format!("/api/voice-requests/{id}/accept")).await;
    let item_id = accepted["grocery_item"]["id"].as_str().unwrap().to_string();
    app.delete(&format!("/api/grocery-items/{item_id}")).await;

    let (status, body) = app
        .post(&format!("/api/voice-requests/{id}/undo-accept"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "pending");
}

#[sqlx::test]
async fn an_accept_says_whether_it_merged(pool: PgPool) {
    let app = TestApp::new(pool);
    let item = unique_item();
    let first = app.create_pending(&item, 1).await;
    let (_, created) = app
        .post(&format!("/api/voice-requests/{first}/accept"))
        .await;
    let second = app.create_pending(&item, 1).await;
    let (_, merged) = app
        .post(&format!(
            "/api/voice-requests/{second}/accept?on_duplicate=merge"
        ))
        .await;

    assert_eq!(created["merged"], false);
    assert_eq!(merged["merged"], true);
}
