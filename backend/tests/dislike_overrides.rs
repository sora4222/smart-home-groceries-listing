//! Overriding a dislike for one order, through the real router.
//!
//! `PUT /api/grocery-items/{id}/dislike-override`, `GET /api/dislike-overrides`
//! and `DELETE /api/grocery-items/{id}/dislike-override/{store}/{product_id}`,
//! plus the dislike book the optimiser will read.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use grocery_backend::services::dislikes::skip::DislikeScope;
use grocery_backend::services::dislikes::DislikeService;
use grocery_backend::services::stores::Store;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

fn milk() -> Value {
    json!({ "store": "woolworths", "product_id": "w-milk-2l", "name": "Full Cream Milk" })
}

fn milk_choice() -> Value {
    json!({ "store": "woolworths", "product_id": "w-milk-2l" })
}

fn override_uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/dislike-override")
}

async fn overrides(app: &TestApp) -> Vec<Value> {
    let (status, body) = app.get("/api/dislike-overrides").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().expect("an array").clone()
}

#[sqlx::test]
async fn overriding_keeps_the_dislike_and_saves_the_override(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json("/api/product-dislikes", &milk()).await;

    let (status, body) = app.put_json(&override_uri(&id), &milk_choice()).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["grocery_item_id"], id);
    assert_eq!(body["store"], "woolworths");
    assert_eq!(body["product_id"], "w-milk-2l");
    assert_eq!(body["overridden_by"], "dev-user");
    assert_eq!(overrides(&app).await.len(), 1);
    let (_, dislikes) = app.get("/api/product-dislikes").await;
    assert_eq!(dislikes.as_array().unwrap().len(), 1, "the dislike stays");
}

#[sqlx::test]
async fn overriding_twice_keeps_one_override(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json("/api/product-dislikes", &milk()).await;

    app.put_json(&override_uri(&id), &milk_choice()).await;
    let (status, _) = app.put_json(&override_uri(&id), &milk_choice()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(overrides(&app).await.len(), 1);
}

#[sqlx::test]
async fn a_product_nobody_dislikes_cannot_be_overridden(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    let (status, body) = app.put_json(&override_uri(&id), &milk_choice()).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(overrides(&app).await.is_empty());
}

#[sqlx::test]
async fn an_unknown_item_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    app.put_json("/api/product-dislikes", &milk()).await;
    let missing = Uuid::new_v4().to_string();

    let (status, _) = app.put_json(&override_uri(&missing), &milk_choice()).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_ordered_item_cannot_be_overridden(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json("/api/product-dislikes", &milk()).await;
    sqlx::query("UPDATE grocery_items SET status = 'ordered' WHERE id = $1")
        .bind(Uuid::parse_str(&id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = app.put_json(&override_uri(&id), &milk_choice()).await;

    assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[sqlx::test]
async fn clearing_the_override_makes_the_dislike_count_again(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json("/api/product-dislikes", &milk()).await;
    app.put_json(&override_uri(&id), &milk_choice()).await;

    let (status, _) = app
        .delete(&format!("{}/woolworths/w-milk-2l", override_uri(&id)))
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(overrides(&app).await.is_empty());
}

#[sqlx::test]
async fn removing_the_item_removes_its_overrides(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json("/api/product-dislikes", &milk()).await;
    app.put_json(&override_uri(&id), &milk_choice()).await;

    let (status, _) = app.delete(&format!("/api/grocery-items/{id}")).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(overrides(&app).await.is_empty());
}

#[sqlx::test]
async fn the_book_for_an_item_holds_only_that_items_overrides(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let milk_id = app.add_item(&json!({ "name": "milk" })).await;
    let other_id = app.add_item(&json!({ "name": "bread" })).await;
    app.put_json("/api/product-dislikes", &milk()).await;
    app.put_json(&override_uri(&milk_id), &milk_choice()).await;

    let service = DislikeService::new(&pool);
    let overridden = service
        .book_for_item(Uuid::parse_str(&milk_id).unwrap())
        .await
        .unwrap();
    let other = service
        .book_for_item(Uuid::parse_str(&other_id).unwrap())
        .await
        .unwrap();

    let scope = DislikeScope::Household;
    assert!(!overridden.skips(Store::Woolworths, "w-milk-2l", scope));
    assert!(other.skips(Store::Woolworths, "w-milk-2l", scope));
    assert_eq!(
        other.disliked_by(Store::Woolworths, "w-milk-2l", scope),
        ["dev"]
    );
}

#[sqlx::test]
async fn overrides_need_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let id = Uuid::new_v4().to_string();
    let (status, _) = app.get("/api/dislike-overrides").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app.put_json(&override_uri(&id), &milk_choice()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
