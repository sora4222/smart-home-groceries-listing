//! The OpenAI-compatible classifier through the real router. `wiremock`
//! stands in for the provider; no test reaches OpenAI or Ollama.

mod common;

use axum::http::StatusCode;
use common::settings::triage_settings;
use common::triage::Shown;
use common::TestApp;
use grocery_backend::config::TriageProvider;
use serde_json::{json, Value};
use sqlx::PgPool;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A chat-completions answer whose message is `content`.
fn completion(content: &str) -> Value {
    json!({ "choices": [{ "index": 0, "message": { "role": "assistant", "content": content } }] })
}

/// Serves `answer` to every chat-completions call, `calls` times exactly.
async fn provider(answer: ResponseTemplate, calls: u64) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(answer)
        .expect(calls)
        .mount(&server)
        .await;
    server
}

fn app(pool: PgPool, server: &MockServer) -> TestApp {
    let base_url = format!("{}/v1", server.uri());
    TestApp::with_triage(pool, triage_settings(TriageProvider::OpenAi, &base_url))
}

#[sqlx::test]
async fn sends_the_model_key_and_item_and_reads_the_answer(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "model": "test-model",
            "response_format": { "type": "json_object" },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion(
            r#"{"supermarket_item": true, "confidence": 0.97, "reason": "Milk is a grocery."}"#,
        )))
        .expect(1)
        .mount(&server)
        .await;
    let app = app(pool, &server);

    let (_, shown, request) = app.webhook_item_triaged("milk").await;

    assert_eq!(shown, Shown::Pending);
    assert_eq!(request["triage_reason"], "Milk is a grocery.");
    let sent = &server.received_requests().await.unwrap()[0];
    let body: Value = serde_json::from_slice(&sent.body).unwrap();
    assert_eq!(body["messages"][1]["content"], r#"{"item":"milk"}"#);
}

#[sqlx::test]
async fn a_low_confidence_answer_is_held_for_review(pool: PgPool) {
    let server = provider(
        ResponseTemplate::new(200).set_body_json(completion(
            r#"{"supermarket_item": true, "confidence": 0.4, "reason": "Not sure what this is."}"#,
        )),
        1,
    )
    .await;
    let app = app(pool, &server);

    let (_, shown, request) = app.webhook_item_triaged("flibber").await;

    assert_eq!(shown, Shown::Held);
    assert_eq!(request["triage_status"], "held");
    assert_eq!(request["triage_reason"], "Not sure what this is.");
}

#[sqlx::test]
async fn a_provider_error_is_held_with_a_plain_reason(pool: PgPool) {
    let server = provider(
        ResponseTemplate::new(500).set_body_string("secret details"),
        1,
    )
    .await;
    let app = app(pool, &server);

    let (_, shown, request) = app.webhook_item_triaged("milk").await;

    assert_eq!(shown, Shown::Held);
    assert_eq!(
        request["triage_reason"],
        "The checker could not be reached (answered 500)"
    );
}

#[sqlx::test]
async fn an_unreadable_answer_is_held(pool: PgPool) {
    let server = provider(
        ResponseTemplate::new(200).set_body_json(completion("Sure! Milk is groceries.")),
        1,
    )
    .await;
    let app = app(pool, &server);

    let (_, shown, _) = app.webhook_item_triaged("milk").await;

    assert_eq!(shown, Shown::Held);
}

#[sqlx::test]
async fn a_held_item_raises_the_held_count_for_the_badge(pool: PgPool) {
    let server = provider(ResponseTemplate::new(503), 2).await;
    let app = app(pool, &server);

    app.webhook_item_triaged("milk").await;
    app.webhook_item_triaged("bread").await;

    let (status, held) = app.get("/api/triage?tab=held").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(held.as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn an_alexa_retry_is_not_classified_again(pool: PgPool) {
    let server = provider(
        ResponseTemplate::new(200).set_body_json(completion(
            r#"{"supermarket_item": true, "confidence": 0.9, "reason": "Rice."}"#,
        )),
        1,
    )
    .await;
    let app = app(pool, &server);
    let body = json!({ "item": "rice", "external_id": "amzn-retry" });

    let (_, first) = app.post_alexa(&body).await;
    app.triaged(first["id"].as_str().unwrap()).await;
    let (status, _) = app.post_alexa(&body).await;

    assert_eq!(status, StatusCode::OK);
    // `expect(1)` on the mock fails the test when the server drops if a
    // second classification was made.
}
