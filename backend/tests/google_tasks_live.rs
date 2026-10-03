//! The real Google Tasks client against `wiremock` standing in for Google:
//! the OAuth calls, the Tasks API calls, and what a poll does when Google
//! refuses. Never the real Google.

mod common;

use axum::http::StatusCode;
use common::{google_tasks_settings, triage_settings, TestApp};
use grocery_backend::config::{GoogleTasksMode, TriageProvider};
use serde_json::json;
use sqlx::PgPool;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const LIST: &str = "/tasks/v1/lists/groceries/tasks";

/// Google's token endpoint: a code or the refresh token both work.
async fn google(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .and(body_string_contains("code=good-code"))
        .and(body_string_contains("client_secret=test-client-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "access-1", "expires_in": 3599,
            "refresh_token": "refresh-1", "token_type": "Bearer"
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=refresh-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "access-1", "expires_in": 3599, "token_type": "Bearer"
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tasks/v1/users/@me/lists"))
        .and(header("authorization", "Bearer access-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{ "id": "groceries", "title": "Groceries" }]
        })))
        .mount(server)
        .await;
}

/// Open tasks on the Groceries list.
async fn tasks_on_list(server: &MockServer, tasks: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(LIST))
        .and(query_param("showCompleted", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "items": tasks })))
        .mount(server)
        .await;
}

async fn watching(pool: PgPool, server: &MockServer) -> TestApp {
    google(server).await;
    let app = TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Live, &server.uri()),
        triage_settings(TriageProvider::Off, ""),
    );
    app.connect_google_tasks("good-code").await;
    app.watch_list("groceries").await;
    app
}

#[sqlx::test]
async fn the_sign_in_url_is_googles_with_the_tasks_scope(pool: PgPool) {
    let server = MockServer::start().await;
    let app = TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Live, &server.uri()),
        triage_settings(TriageProvider::Off, ""),
    );

    let (_, started) = app.post("/api/intake/google-tasks/sign-in").await;

    let url = started["authorize_url"].as_str().unwrap();
    assert!(url.starts_with(&format!("{}/o/oauth2/v2/auth?", server.uri())));
    assert!(url.contains("scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Ftasks"));
    assert!(url.contains("access_type=offline"));
}

#[sqlx::test]
async fn a_poll_records_then_deletes_each_task(pool: PgPool) {
    let server = MockServer::start().await;
    let app = watching(pool, &server).await;
    tasks_on_list(
        &server,
        json!([{ "id": "t1", "title": "3 apples", "status": "needsAction" }]),
    )
    .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{LIST}/t1")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let (status, report) = app.poll_google_tasks().await;

    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["recorded"], 1);
    assert_eq!(report["not_deleted"], 0);
    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending[0]["parsed_name"], "apples");
    assert_eq!(pending[0]["parsed_quantity"], 3);
}

#[sqlx::test]
async fn a_failed_delete_is_retried_without_a_second_card(pool: PgPool) {
    let server = MockServer::start().await;
    let app = watching(pool, &server).await;
    tasks_on_list(&server, json!([{ "id": "t1", "title": "milk" }])).await;
    Mock::given(method("DELETE"))
        .and(path(format!("{LIST}/t1")))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let (_, first) = app.poll_google_tasks().await;
    let (_, second) = app.poll_google_tasks().await;

    assert_eq!(first["recorded"], 1);
    assert_eq!(first["not_deleted"], 1);
    assert_eq!(second["recorded"], 0);
    assert_eq!(second["already_seen"], 1);
    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending.as_array().unwrap().len(), 1);
    let (_, settings) = app.get("/api/intake/settings").await;
    assert!(settings["google_tasks"]["last_poll_error"]
        .as_str()
        .unwrap()
        .contains("could not be deleted"));
}

#[sqlx::test]
async fn nothing_is_deleted_when_google_cannot_be_read(pool: PgPool) {
    let server = MockServer::start().await;
    let app = watching(pool, &server).await;
    Mock::given(method("GET"))
        .and(path(LIST))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;

    let (status, body) = app.poll_google_tasks().await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["detail"].as_str().unwrap().contains("answered 503"));
    let (_, settings) = app.get("/api/intake/settings").await;
    assert!(settings["google_tasks"]["last_poll_error"]
        .as_str()
        .unwrap()
        .contains("could not be reached"));
}

#[sqlx::test]
async fn a_revoked_sign_in_says_to_connect_again(pool: PgPool) {
    let server = MockServer::start().await;
    let app = watching(pool, &server).await;
    // The household removed the app's access in their Google account.
    server.reset().await;
    Mock::given(method("GET"))
        .and(path(LIST))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "error": "invalid_grant" })))
        .mount(&server)
        .await;

    // First the cached access token is refused, then the refresh token.
    let (first, _) = app.poll_google_tasks().await;
    let (second, body) = app.poll_google_tasks().await;

    assert_eq!(first, StatusCode::CONFLICT);
    assert_eq!(second, StatusCode::CONFLICT);
    assert!(body["detail"]
        .as_str()
        .unwrap()
        .contains("connect Google Tasks again"));
    let (_, settings) = app.get("/api/intake/settings").await;
    assert!(settings["google_tasks"]["last_poll_error"]
        .as_str()
        .unwrap()
        .contains("connect Google Tasks again"));
}
