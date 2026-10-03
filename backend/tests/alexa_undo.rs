//! Integration tests for "Alexa, undo": reversing a voice remove or reduce.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{TestApp, TEST_BRIDGE_SECRET};
use grocery_backend::routes::alexa::BRIDGE_SECRET_HEADER;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// `POST /api/intake/alexa/<action>` as the bridge sidecar.
async fn bridge_post(app: &TestApp, action: &str, body: &Value) -> (StatusCode, Value) {
    app.send(
        Request::builder()
            .method(Method::POST)
            .uri(format!("/api/intake/alexa/{action}"))
            .header("content-type", "application/json")
            .header(BRIDGE_SECRET_HEADER, TEST_BRIDGE_SECRET)
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
}

async fn remove(app: &TestApp, body: &Value) -> (StatusCode, Value) {
    bridge_post(app, "remove", body).await
}

async fn undo(app: &TestApp, external_id: &str) -> (StatusCode, Value) {
    bridge_post(app, "undo", &json!({ "external_id": external_id })).await
}

fn request_id() -> String {
    format!("amzn1.echo-api.request.{}", Uuid::new_v4())
}

/// The list as JSON, newest first.
async fn items(app: &TestApp) -> Vec<Value> {
    let (_, items) = app.get("/api/grocery-items").await;
    items.as_array().unwrap().clone()
}

#[sqlx::test]
async fn undo_puts_a_removed_item_back_as_it_was(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({
            "name": "milk", "quantity": 2, "note": "the blue one", "filter_terms": ["lactose free"]
        }))
        .await;
    let before = items(&app).await;
    remove(&app, &json!({ "item": "milk" })).await;

    let (status, body) = undo(&app, &request_id()).await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["kind"], "removed");
    assert_eq!(body["undone"], true);
    let after = items(&app).await;
    assert_eq!(after, before, "same id, quantity, note, chips and place");
    assert_eq!(after[0]["id"], id);
}

#[sqlx::test]
async fn undo_adds_back_what_a_reduce_took_off(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "eggs", "quantity": 6 }))
        .await;
    remove(&app, &json!({ "item": "eggs", "quantity": 2 })).await;
    // Someone edits the quantity in the web app in between; that is kept.
    app.patch_json(
        &format!("/api/grocery-items/{id}"),
        &json!({ "quantity": 5 }),
    )
    .await;

    let (status, _) = undo(&app, &request_id()).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(items(&app).await[0]["quantity"], 7);
}

#[sqlx::test]
async fn undo_again_reverses_the_change_before(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice" })).await;
    app.add_item(&json!({ "name": "pasta" })).await;
    remove(&app, &json!({ "item": "rice" })).await;
    remove(&app, &json!({ "item": "pasta" })).await;

    let (_, first) = undo(&app, &request_id()).await;
    let (_, second) = undo(&app, &request_id()).await;
    let (third_status, _) = undo(&app, &request_id()).await;

    assert_eq!(first["item_name"], "pasta");
    assert_eq!(second["item_name"], "rice");
    assert_eq!(third_status, StatusCode::NOT_FOUND, "nothing left to undo");
    assert_eq!(items(&app).await.len(), 2);
}

#[sqlx::test]
async fn a_retried_undo_does_not_reverse_a_second_change(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice" })).await;
    app.add_item(&json!({ "name": "pasta" })).await;
    remove(&app, &json!({ "item": "rice" })).await;
    remove(&app, &json!({ "item": "pasta" })).await;
    let undo_id = request_id();

    let (first_status, first) = undo(&app, &undo_id).await;
    let (retry_status, retry) = undo(&app, &undo_id).await;

    assert_eq!(first_status, StatusCode::CREATED);
    assert_eq!(retry_status, StatusCode::OK);
    assert_eq!(first["id"], retry["id"]);
    let names: Vec<_> = items(&app)
        .await
        .iter()
        .map(|i| i["name"].clone())
        .collect();
    assert_eq!(names, vec![json!("pasta")], "rice stays removed");
}

#[sqlx::test]
async fn with_nothing_to_undo_it_is_a_404(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = undo(&app, &request_id()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_undo_with_no_body_still_works(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice" })).await;
    remove(&app, &json!({ "item": "rice" })).await;

    let (status, _) = app
        .send(
            Request::builder()
                .method(Method::POST)
                .uri("/api/intake/alexa/undo")
                .header(BRIDGE_SECRET_HEADER, TEST_BRIDGE_SECRET)
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(items(&app).await.len(), 1);
}

#[sqlx::test]
async fn a_change_older_than_the_window_is_settled(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    app.add_item(&json!({ "name": "rice" })).await;
    remove(&app, &json!({ "item": "rice" })).await;
    sqlx::query("UPDATE voice_list_changes SET created_at = now() - interval '31 minutes'")
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = undo(&app, &request_id()).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(items(&app).await.is_empty());
}

#[sqlx::test]
async fn a_reduced_item_deleted_since_cannot_be_changed_back(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "eggs", "quantity": 6 }))
        .await;
    remove(&app, &json!({ "item": "eggs", "quantity": 1 })).await;
    app.delete(&format!("/api/grocery-items/{id}")).await;

    let (status, _) = undo(&app, &request_id()).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(items(&app).await.is_empty());
}

#[sqlx::test]
async fn undo_requires_the_bridge_secret(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app
        .send(
            Request::builder()
                .method(Method::POST)
                .uri("/api/intake/alexa/undo")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
