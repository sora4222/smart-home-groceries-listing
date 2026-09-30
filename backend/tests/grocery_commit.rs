//! Committing the reviewed list for purchase, and releasing it again.
//!
//! The commit is the point the household stops editing and the order flow
//! takes over, so the rule under test is that a committed item is locked.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn committing_marks_every_active_item(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk" })).await;
    app.add_item(&json!({ "name": "bread" })).await;

    let (status, committed) = app.post("/api/grocery-items/commit").await;

    assert_eq!(status, StatusCode::OK);
    let items = committed.as_array().expect("an array");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| item["status"] == "committed"));

    // Committed items stay on the list so the web app can show what is locked.
    let (_, listed) = app.get("/api/grocery-items").await;
    let listed = listed.as_array().expect("an array");
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|item| item["status"] == "committed"));
}

#[sqlx::test]
async fn committing_an_empty_list_changes_nothing(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, committed) = app.post("/api/grocery-items/commit").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(committed.as_array().expect("an array").len(), 0);
}

#[sqlx::test]
async fn committing_twice_is_harmless(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, second) = app.post("/api/grocery-items/commit").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(second.as_array().expect("an array").len(), 0);
}

#[sqlx::test]
async fn a_committed_item_cannot_be_edited(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "quantity": 9 }),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["detail"]
        .as_str()
        .expect("a message")
        .contains("committed"));
}

#[sqlx::test]
async fn a_committed_item_cannot_be_deleted(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, _) = app.delete(&format!("/api/grocery-items/{id}")).await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn releasing_reopens_the_list_for_editing(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, released) = app.post("/api/grocery-items/release").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(released.as_array().expect("an array").len(), 1);
    assert_eq!(released[0]["status"], "active");

    let (edited, _) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "quantity": 9 }),
        )
        .await;
    assert_eq!(edited, StatusCode::OK);
}

#[sqlx::test]
async fn a_new_item_added_after_a_commit_is_still_editable(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;

    // A committed item does not absorb a later request, so the same name can
    // start the next list.
    let id = app
        .add_item(&json!({ "name": "milk", "quantity": 2 }))
        .await;

    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "quantity": 3 }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["quantity"], 3);
    assert_eq!(body["status"], "active");
}

#[sqlx::test]
async fn a_browser_without_a_session_cannot_commit(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    for (status, _) in [
        app.post("/api/grocery-items/commit").await,
        app.post("/api/grocery-items/release").await,
    ] {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
