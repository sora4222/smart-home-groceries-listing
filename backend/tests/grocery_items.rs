//! The grocery list's own API: read, add, modify, annotate, remove.
//!
//! Each test drives the real router over its own database. Committing the list
//! for purchase is covered separately in `tests/grocery_commit.rs`.

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
            "/api/grocery-items?merge=true",
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
            "/api/grocery-items?merge=true",
            &json!({ "name": "rice", "quantity": 5 }),
        )
        .await;

    assert_eq!(body["quantity"], 999);
}

#[sqlx::test]
async fn an_edit_changes_name_quantity_note_and_chips(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({
                "name": "full cream milk",
                "quantity": 3,
                "note": "the 2 litre bottle",
                "filter_terms": ["full cream"]
            }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "full cream milk");
    assert_eq!(body["quantity"], 3);
    assert_eq!(body["note"], "the 2 litre bottle");
    assert_eq!(body["filter_terms"], json!(["full cream"]));
}

#[sqlx::test]
async fn an_edit_leaves_out_what_it_does_not_mention(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(
            &json!({ "name": "milk", "quantity": 2, "note": "keep me", "filter_terms": ["a2"] }),
        )
        .await;

    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "quantity": 4 }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["quantity"], 4);
    assert_eq!(body["name"], "milk");
    assert_eq!(body["note"], "keep me");
    assert_eq!(body["filter_terms"], json!(["a2"]));
}

#[sqlx::test]
async fn an_empty_note_clears_the_annotation(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "milk", "note": "remove me" }))
        .await;

    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "note": "   " }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["note"], json!(null));
}

#[sqlx::test]
async fn sending_the_remaining_chips_is_how_one_is_removed(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "toilet paper", "filter_terms": ["3 ply", "recycled"] }))
        .await;

    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "filter_terms": ["recycled"] }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["filter_terms"], json!(["recycled"]));

    let (_, cleared) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "filter_terms": [] }),
        )
        .await;
    assert_eq!(cleared["filter_terms"], json!([]));
}

#[sqlx::test]
async fn editing_an_item_that_does_not_exist_is_a_404(pool: PgPool) {
    let app = TestApp::new(pool);
    let missing = uuid::Uuid::new_v4();

    let (status, _) = app
        .patch_json(
            &format!("/api/grocery-items/{missing}"),
            &json!({ "quantity": 2 }),
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_invalid_edit_is_refused_before_it_reaches_the_row(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "milk", "quantity": 2 }))
        .await;

    let (status, _) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "quantity": 0 }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (_, listed) = app.get("/api/grocery-items").await;
    assert_eq!(listed[0]["quantity"], 2);
}

#[sqlx::test]
async fn a_deleted_item_leaves_the_list(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    let (status, _) = app.delete(&format!("/api/grocery-items/{id}")).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, listed) = app.get("/api/grocery-items").await;
    assert_eq!(listed.as_array().expect("an array").len(), 0);
}

#[sqlx::test]
async fn deleting_an_item_that_does_not_exist_is_a_404(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .delete(&format!("/api/grocery-items/{}", uuid::Uuid::new_v4()))
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_item_a_voice_request_produced_can_still_be_deleted(pool: PgPool) {
    let app = TestApp::new(pool);
    let request_id = app.create_pending("oat milk", 1).await;
    let (_, accepted) = app
        .post_json(
            &format!("/api/voice-requests/{request_id}/accept"),
            &json!({}),
        )
        .await;
    let item_id = accepted["grocery_item"]["id"].as_str().expect("an id");

    // The accepted request points at this row; clearing that reference rather
    // than blocking the delete is what migration 0002's FK change is for.
    let (status, _) = app.delete(&format!("/api/grocery-items/{item_id}")).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, listed) = app.get("/api/grocery-items").await;
    assert_eq!(listed.as_array().expect("an array").len(), 0);
}

#[sqlx::test]
async fn a_browser_without_a_session_gets_nothing(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    for (status, _) in [
        app.get("/api/grocery-items").await,
        app.post_json("/api/grocery-items", &json!({ "name": "milk" }))
            .await,
        app.patch_json(
            &format!("/api/grocery-items/{}", uuid::Uuid::new_v4()),
            &json!({ "quantity": 2 }),
        )
        .await,
        app.delete(&format!("/api/grocery-items/{}", uuid::Uuid::new_v4()))
            .await,
    ] {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
