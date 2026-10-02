//! Network access logging through the real router: what each request leaves
//! in `access_logs`, and the `/logs` page's `GET /api/access-logs`.

mod common;

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use common::access_logs::{logged_requests, wait_for_logged, wait_for_path};
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

fn get_request(uri: &str) -> axum::http::request::Builder {
    Request::builder().method(Method::GET).uri(uri)
}

#[sqlx::test]
async fn a_signed_in_request_is_logged_with_its_user(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    let (status, _) = app.get("/api/grocery-items").await;
    assert_eq!(status, StatusCode::OK);

    let row = wait_for_path(&pool, "/api/grocery-items").await;
    assert_eq!(row.user_id, "dev-user");
    assert_eq!(row.method, "GET");
    assert_eq!(row.status_code, 200);
}

#[sqlx::test]
async fn a_request_without_a_session_is_logged_as_unauthenticated(pool: PgPool) {
    let app = TestApp::with_auth(pool.clone());

    let (status, _) = app.get("/api/grocery-items").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let row = wait_for_path(&pool, "/api/grocery-items").await;
    assert_eq!(row.user_id, "unauthenticated");
    assert_eq!(row.status_code, 401);
}

#[sqlx::test]
async fn an_intake_webhook_is_logged_as_unauthenticated(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    let (status, _) = app.post_webhook(&json!({ "item": "milk" })).await;
    assert_eq!(status, StatusCode::CREATED);

    let row = wait_for_path(&pool, "/api/voice-requests").await;
    assert_eq!(row.user_id, "unauthenticated");
    assert_eq!(row.method, "POST");
    assert_eq!(row.status_code, 201);
}

#[sqlx::test]
async fn an_unknown_path_is_logged_too(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    let (status, _) = app.get("/wp-admin/login.php").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let row = wait_for_path(&pool, "/wp-admin/login.php").await;
    assert_eq!(row.status_code, 404);
}

#[sqlx::test]
async fn the_query_string_is_never_stored(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    app.get("/api/triage?tab=held&secret=hunter2").await;

    let row = wait_for_path(&pool, "/api/triage").await;
    assert!(logged_requests(&pool)
        .await
        .iter()
        .all(|r| !r.path.contains("hunter2")));
    assert_eq!(row.path, "/api/triage");
}

#[sqlx::test]
async fn the_connection_address_and_forwarded_header_are_stored(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let mut request = get_request("/api/health")
        .header("cf-connecting-ip", "203.0.113.9")
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(ConnectInfo(
        "172.18.0.5:40000".parse::<SocketAddr>().unwrap(),
    ));

    app.send(request).await;

    let row = wait_for_path(&pool, "/api/health").await;
    assert_eq!(row.source_ip.as_deref(), Some("172.18.0.5"));
    assert_eq!(row.forwarded_for.as_deref(), Some("203.0.113.9"));
}

#[sqlx::test]
async fn without_connection_info_the_address_is_empty(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    app.get("/api/health").await;

    let row = wait_for_path(&pool, "/api/health").await;
    assert_eq!(row.source_ip, None);
    assert_eq!(row.forwarded_for, None);
}

#[sqlx::test]
async fn the_log_page_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (status, _) = app.get("/api/access-logs").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn the_log_page_lists_newest_first_and_hides_health_checks(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    app.get("/api/grocery-items").await;
    wait_for_logged(&pool, 1).await;
    app.get("/api/health").await;
    wait_for_logged(&pool, 2).await;
    app.get("/api/item-rules").await;
    wait_for_logged(&pool, 3).await;

    let (status, page) = app.get("/api/access-logs").await;
    assert_eq!(status, StatusCode::OK);
    let paths: Vec<&str> = page["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, ["/api/item-rules", "/api/grocery-items"]);
    assert_eq!(page["next_before"], json!(null));
    let newest = &page["entries"][0];
    assert_eq!(newest["user_id"], "dev-user");
    assert_eq!(newest["method"], "GET");
    assert_eq!(newest["status_code"], 200);
    assert!(newest["occurred_at"].is_string());
}

#[sqlx::test]
async fn health_checks_show_when_asked(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    app.get("/api/health").await;
    wait_for_logged(&pool, 1).await;

    let (_, page) = app.get("/api/access-logs?hide_health_checks=false").await;

    assert_eq!(page["entries"][0]["path"], "/api/health");
}

#[sqlx::test]
async fn the_log_page_pages_back_through_older_rows(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    for path in ["/api/grocery-items", "/api/item-rules", "/api/order-review"] {
        app.get(path).await;
    }
    wait_for_logged(&pool, 3).await;

    let (_, first) = app.get("/api/access-logs?limit=2").await;
    assert_eq!(first["entries"].as_array().unwrap().len(), 2);
    let before = first["next_before"]
        .as_i64()
        .expect("a full page has a cursor");

    let (_, second) = app
        .get(&format!("/api/access-logs?limit=2&before={before}"))
        .await;
    let older = second["entries"].as_array().unwrap();
    assert!(older.iter().all(|e| e["id"].as_i64().unwrap() < before));
    assert!(!older.is_empty());
}

#[sqlx::test]
async fn a_bad_page_parameter_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.get("/api/access-logs?limit=lots").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn an_oversized_body_is_logged_with_its_refusal(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let huge = json!({ "name": "x".repeat(64 * 1024) });

    let (status, _) = app.post_json("/api/grocery-items", &huge).await;
    assert!(status.is_client_error(), "{status}");

    let row = wait_for_path(&pool, "/api/grocery-items").await;
    assert_eq!(row.status_code, i16::try_from(status.as_u16()).unwrap());
    assert_eq!(row.method, "POST");
}
