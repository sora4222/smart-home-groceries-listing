//! The local model servers — Ollama, llama.cpp's `llama-server` and vLLM —
//! through the real router. Each serves OpenAI's chat-completions API, so
//! they share one client; these tests check each provider setting reaches its
//! server with the right model and no key, and that each server's own answer
//! (`tests/fixtures/triage/<server>.json`, the shape each one returns) is
//! read. `wiremock` stands in for the server; no model runs here.

mod common;

use common::triage::Shown;
use common::{triage_settings, TestApp};
use grocery_backend::config::{TriageProvider, TriageSettings};
use serde_json::Value;
use sqlx::PgPool;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// One server's recorded answer.
fn answer(server: &str) -> Value {
    let raw = std::fs::read_to_string(format!(
        "{}/tests/fixtures/triage/{server}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("the fixture should exist");
    serde_json::from_str(&raw).expect("the fixture should be JSON")
}

/// A local server answering with its own recorded reply, once, and only when
/// asked for `model`.
async fn local_server(fixture: &str, model: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(serde_json::json!({
            "model": model,
            "response_format": { "type": "json_object" },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer(fixture)))
        .expect(1)
        .mount(&server)
        .await;
    server
}

/// Settings for a local server with no key, as most home setups run.
fn local(provider: TriageProvider, server: &MockServer, model: &str) -> TriageSettings {
    TriageSettings {
        model: model.to_string(),
        api_key: String::new(),
        ..triage_settings(provider, &format!("{}/v1", server.uri()))
    }
}

/// Delivers an item and checks the server approved it, with no key sent.
async fn approves_oat_milk(pool: PgPool, provider: TriageProvider, fixture: &str, model: &str) {
    let server = local_server(fixture, model).await;
    let app = TestApp::with_triage(pool, local(provider, &server, model));

    let (_, shown, request) = app.webhook_item_triaged("oat milk").await;

    assert_eq!(shown, Shown::Pending, "{request}");
    assert_eq!(request["triage_status"], "approved");
    assert!(request["triage_reason"]
        .as_str()
        .unwrap()
        .contains("Oat milk"));
    let sent = &server.received_requests().await.unwrap()[0];
    assert!(
        !sent.headers.contains_key("authorization"),
        "a local server without --api-key gets no key"
    );
}

#[sqlx::test]
async fn ollama_answers_are_read(pool: PgPool) {
    approves_oat_milk(pool, TriageProvider::Ollama, "ollama", "llama3.2").await;
}

#[sqlx::test]
async fn llama_cpp_answers_are_read(pool: PgPool) {
    approves_oat_milk(pool, TriageProvider::LlamaCpp, "llamacpp", "local").await;
}

#[sqlx::test]
async fn vllm_answers_are_read(pool: PgPool) {
    approves_oat_milk(
        pool,
        TriageProvider::Vllm,
        "vllm",
        "Qwen/Qwen2.5-1.5B-Instruct",
    )
    .await;
}

#[sqlx::test]
async fn a_local_server_started_with_a_key_gets_it(pool: PgPool) {
    let server = local_server("vllm", "my-model").await;
    let settings = TriageSettings {
        api_key: "local-key".to_string(),
        ..local(TriageProvider::Vllm, &server, "my-model")
    };
    let app = TestApp::with_triage(pool, settings);

    app.webhook_item_triaged("oat milk").await;

    let sent = &server.received_requests().await.unwrap()[0];
    assert_eq!(
        sent.headers.get("authorization").unwrap(),
        "Bearer local-key"
    );
}

#[sqlx::test]
async fn a_local_server_that_is_not_running_holds_the_item(pool: PgPool) {
    // Port 9 (discard) refuses connections, like a server that is not started.
    let settings = TriageSettings {
        api_key: String::new(),
        ..triage_settings(TriageProvider::LlamaCpp, "http://127.0.0.1:9/v1")
    };
    let app = TestApp::with_triage(pool, settings);

    let (_, shown, request) = app.webhook_item_triaged("oat milk").await;

    assert_eq!(shown, Shown::Held);
    assert_eq!(
        request["triage_reason"],
        "The checker could not be reached (no connection)"
    );
}
