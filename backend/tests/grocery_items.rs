//! Reading the list and adding to it, including the spec's duplicate handling.
//!
//! Each test drives the real router over its own database. Changing and
//! removing an item is `tests/grocery_item_edits.rs`; committing the list for
//! purchase is `tests/grocery_commit.rs`.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn the_list_starts_empty(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get("/api/grocery-items").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("an array").len(), 0);
}

#[sqlx::test]
async fn an_added_item_is_on_the_list(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, created) = app
        .post_json(
            "/api/grocery-items",
            &json!({ "name": "milk", "quantity": 2 }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "milk");
    assert_eq!(created["quantity"], 2);
    assert_eq!(created["status"], "active");
    assert_eq!(created["source"], "manual");
    assert_eq!(created["note"], json!(null));
    assert_eq!(created["filter_terms"], json!([]));

    let (_, listed) = app.get("/api/grocery-items").await;
    let items = listed.as_array().expect("an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], created["id"]);
}

#[sqlx::test]
async fn quantity_defaults_to_one(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, created) = app
        .post_json("/api/grocery-items", &json!({ "name": "bananas" }))
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["quantity"], 1);
}

#[sqlx::test]
async fn an_item_can_be_annotated_as_it_is_added(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, created) = app
        .post_json(
            "/api/grocery-items",
            &json!({
                "name": "toilet paper",
                "note": "  the recycled one  ",
                "filter_terms": ["  3 ply ", "3 PLY", "", "recycled"]
            }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["note"], "the recycled one");
    // Trimmed, blanks dropped, and a case-insensitive repeat collapsed.
    assert_eq!(created["filter_terms"], json!(["3 ply", "recycled"]));
}

#[sqlx::test]
async fn a_name_that_is_only_whitespace_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .post_json("/api/grocery-items", &json!({ "name": "" }))
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn a_quantity_above_the_column_limit_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .post_json(
            "/api/grocery-items",
            &json!({ "name": "rice", "quantity": 1000 }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn too_many_filter_terms_are_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let terms: Vec<String> = (0..11).map(|n| format!("term {n}")).collect();

    let (status, _) = app
        .post_json(
            "/api/grocery-items",
            &json!({ "name": "rice", "filter_terms": terms }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn a_filter_term_that_is_too_long_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .post_json(
            "/api/grocery-items",
            &json!({ "name": "rice", "filter_terms": ["x".repeat(61)] }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn adding_a_name_already_on_the_list_asks_first(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "Full Cream Milk", "quantity": 1 }))
        .await;

    // Same item, differently spelled: the normalised name is what matches.
    let (status, body) = app
        .post_json(
            "/api/grocery-items",
            &json!({ "name": "  full   cream   MILK ", "quantity": 2 }),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["detail"]["existing_item"]["name"], "Full Cream Milk");
    assert_eq!(body["detail"]["existing_item"]["quantity"], 1);
    assert!(body["detail"]["message"]
        .as_str()
        .expect("a message")
        .contains("already on the list"));

    let (_, listed) = app.get("/api/grocery-items").await;
    assert_eq!(listed.as_array().expect("an array").len(), 1);
}

#[sqlx::test]
async fn merge_combines_the_quantities_instead_of_adding_a_second_entry(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 1 }))
        .await;

    let (status, body) = app
        .post_json(
            "/api/grocery-items?on_duplicate=merge",
            &json!({ "name": "milk", "quantity": 2 }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["quantity"], 3);

    let (_, listed) = app.get("/api/grocery-items").await;
    assert_eq!(listed.as_array().expect("an array").len(), 1);
}

#[sqlx::test]
async fn merging_stops_at_the_column_maximum(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice", "quantity": 998 }))
        .await;

    let (_, body) = app
        .post_json(
            "/api/grocery-items?on_duplicate=merge",
            &json!({ "name": "rice", "quantity": 5 }),
        )
        .await;

    assert_eq!(body["quantity"], 999);
}

#[sqlx::test]
async fn a_duplicate_can_be_kept_as_a_separate_entry(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "note": "full cream" }))
        .await;

    let (status, body) = app
        .post_json(
            "/api/grocery-items?on_duplicate=separate",
            &json!({ "name": "milk", "note": "oat, for Sam" }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["note"], "oat, for Sam");

    let (_, listed) = app.get("/api/grocery-items").await;
    assert_eq!(listed.as_array().expect("an array").len(), 2);
}

#[sqlx::test]
async fn an_unknown_duplicate_instruction_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .post_json(
            "/api/grocery-items?on_duplicate=whatever",
            &json!({ "name": "milk" }),
        )
        .await;

    assert_ne!(status, StatusCode::CREATED);
}
