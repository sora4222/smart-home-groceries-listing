//! Product dislikes through the real router.
//!
//! `PUT /api/product-dislikes`, `GET` on the same path and
//! `DELETE /api/product-dislikes/{store}/{product_id}`. The router signs
//! every request in as the dev user, so another member's dislike is written
//! straight through the repository.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use grocery_backend::services::dislikes::repository::{self, NewDislike};
use grocery_backend::services::stores::Store;
use serde_json::{json, Value};
use sqlx::PgPool;

fn milk() -> Value {
    json!({
        "store": "woolworths", "product_id": "w-milk-2l",
        "name": "Full Cream Milk", "brand": "Woolworths", "package_size": "2L"
    })
}

/// Phu, another household member, dislikes the same milk.
async fn phu_dislikes_milk(pool: &PgPool) {
    repository::upsert(
        pool,
        &NewDislike {
            user_id: "phu-id",
            user_name: "phu",
            store: Store::Woolworths,
            product_id: "w-milk-2l",
            product_name: "Full Cream Milk",
            brand: Some("Woolworths"),
            package_size: Some("2L"),
        },
    )
    .await
    .expect("saving Phu's dislike");
}

async fn household(app: &TestApp) -> Vec<Value> {
    let (status, body) = app.get("/api/product-dislikes").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().expect("an array").clone()
}

#[sqlx::test]
async fn disliking_a_product_saves_it_as_mine(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.put_json("/api/product-dislikes", &milk()).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["store"], "woolworths");
    assert_eq!(body["store_name"], "Woolworths");
    assert_eq!(body["product_id"], "w-milk-2l");
    assert_eq!(body["name"], "Full Cream Milk");
    assert_eq!(body["brand"], "Woolworths");
    assert_eq!(body["package_size"], "2L");
    assert_eq!(body["user_id"], "dev-user");
    // The dev user's email is dev@example.local.
    assert_eq!(body["user_name"], "dev");
    assert_eq!(body["mine"], true);
    assert!(body["disliked_at"].as_str().is_some());

    assert_eq!(household(&app).await.len(), 1);
}

#[sqlx::test]
async fn disliking_again_keeps_one_row(pool: PgPool) {
    let app = TestApp::new(pool);
    app.put_json("/api/product-dislikes", &milk()).await;

    let mut renamed = milk();
    renamed["name"] = json!("Full Cream Milk 2L");
    let (status, body) = app.put_json("/api/product-dislikes", &renamed).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let all = household(&app).await;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["name"], "Full Cream Milk 2L");
}

#[sqlx::test]
async fn the_household_view_shows_every_members_dislikes(pool: PgPool) {
    phu_dislikes_milk(&pool).await;
    let app = TestApp::new(pool);
    app.put_json("/api/product-dislikes", &milk()).await;

    let all = household(&app).await;

    assert_eq!(all.len(), 2);
    let phu = all.iter().find(|d| d["user_id"] == "phu-id").unwrap();
    assert_eq!(phu["user_name"], "phu");
    assert_eq!(phu["mine"], false);
    let mine = all.iter().find(|d| d["user_id"] == "dev-user").unwrap();
    assert_eq!(mine["mine"], true);
}

#[sqlx::test]
async fn removing_my_dislike_leaves_other_members_dislikes(pool: PgPool) {
    phu_dislikes_milk(&pool).await;
    let app = TestApp::new(pool);
    app.put_json("/api/product-dislikes", &milk()).await;

    let (status, _) = app
        .delete("/api/product-dislikes/woolworths/w-milk-2l")
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    let all = household(&app).await;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["user_id"], "phu-id");
}

#[sqlx::test]
async fn removing_a_dislike_i_never_had_is_not_an_error(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.delete("/api/product-dislikes/coles/c-nothing").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn a_blank_brand_is_saved_as_no_brand(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut body = milk();
    body["brand"] = json!("  ");
    body["package_size"] = Value::Null;

    let (status, saved) = app.put_json("/api/product-dislikes", &body).await;

    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["brand"], Value::Null);
    assert_eq!(saved["package_size"], Value::Null);
}

#[sqlx::test]
async fn a_bad_dislike_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let cases = [
        json!({ "store": "aldi", "product_id": "a", "name": "Milk" }),
        json!({ "store": "coles", "product_id": "", "name": "Milk" }),
        json!({ "store": "coles", "product_id": "a", "name": "" }),
        json!({ "store": "coles", "product_id": "a", "name": "   " }),
        json!({ "store": "coles", "product_id": "a".repeat(101), "name": "Milk" }),
        json!({ "store": "coles", "product_id": "a", "name": "m".repeat(301) }),
        json!({ "store": "coles", "product_id": "a" }),
    ];
    for case in cases {
        let (status, body) = app.put_json("/api/product-dislikes", &case).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{case} → {body}");
    }
    assert!(household(&app).await.is_empty());
}

#[sqlx::test]
async fn dislikes_need_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let (status, _) = app.get("/api/product-dislikes").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app.put_json("/api/product-dislikes", &milk()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app
        .delete("/api/product-dislikes/woolworths/w-milk-2l")
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
