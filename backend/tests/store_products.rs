//! `GET /api/stores/{store}/products/{product_id}` through the real router.
//!
//! The stores are the fake catalogue: an integration test never reaches
//! Woolworths or Coles.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use sqlx::PgPool;

#[sqlx::test]
async fn looks_up_a_product_at_its_store(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get("/api/stores/coles/products/c-milk-3l").await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["product_id"], "c-milk-3l");
    assert_eq!(body["available"], true);
    // Priced for one unit.
    assert_eq!(body["total_price"], body["price"]);
}

#[sqlx::test]
async fn another_stores_product_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.get("/api/stores/woolworths/products/c-milk-3l").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_unknown_store_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.get("/api/stores/aldi/products/123").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}
