//! A classifier behind an OpenAI-compatible chat-completions API.
//!
//! OpenAI and the local model servers — Ollama, llama.cpp's `llama-server`
//! and vLLM — all serve `POST {base}/chat/completions`, so this one client
//! covers them all; [`crate::config::TriageSettings`] picks the base URL,
//! model and key. This is the only file that talks to an LLM provider.

use reqwest::Client;
use serde_json::{json, Value};

use super::model::{Classification, TriageError, TriageModel};
use super::prompt::{first_choice_content, parse_reply, user_message, SYSTEM_PROMPT};
use crate::services::stores::client::BoxFuture;

/// Talks to one OpenAI-compatible endpoint.
pub struct OpenAiCompatible {
    http: Client,
    url: String,
    model: String,
    api_key: String,
}

impl OpenAiCompatible {
    /// A client for `base_url` (e.g. `http://localhost:11434/v1`). An empty
    /// `api_key` sends no `Authorization` header, which is what a local
    /// server started without `--api-key` wants.
    pub fn new(http: Client, base_url: &str, model: &str, api_key: &str) -> Self {
        Self {
            http,
            url: format!("{}/chat/completions", base_url.trim_end_matches('/')),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// The request body for one item: deterministic, JSON-only answers.
    fn body(&self, item_text: &str) -> Value {
        json!({
            "model": self.model,
            "temperature": 0,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user", "content": user_message(item_text) },
            ],
        })
    }

    /// Sends one item and returns the provider's whole JSON answer.
    async fn ask(&self, item_text: &str) -> Result<Value, TriageError> {
        let mut request = self.http.post(&self.url).json(&self.body(item_text));
        if !self.api_key.is_empty() {
            request = request.bearer_auth(&self.api_key);
        }
        let response = request.send().await.map_err(|err| {
            TriageError::Unavailable(
                if err.is_timeout() {
                    "timed out"
                } else {
                    "no connection"
                }
                .into(),
            )
        })?;
        let status = response.status();
        if !status.is_success() {
            return Err(TriageError::Unavailable(format!(
                "answered {}",
                status.as_u16()
            )));
        }
        response.json().await.map_err(|_| TriageError::BadReply)
    }
}

impl TriageModel for OpenAiCompatible {
    fn classify<'a>(
        &'a self,
        item_text: &'a str,
    ) -> BoxFuture<'a, Result<Classification, TriageError>> {
        Box::pin(async move {
            let answer = self.ask(item_text).await?;
            parse_reply(first_choice_content(&answer)?)
        })
    }
}
