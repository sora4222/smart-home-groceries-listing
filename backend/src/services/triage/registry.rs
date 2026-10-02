//! Choosing the classifier for this process. The one place that names a
//! concrete [`TriageModel`].

use std::sync::Arc;

use super::assessor::Triage;
use super::fake::FakeTriage;
use super::model::{Classification, TriageError, TriageModel};
use super::openai::OpenAiCompatible;
use crate::config::{TriageProvider, TriageSettings};
use crate::services::stores::client::BoxFuture;

/// Builds the classifier `settings` asks for.
pub fn build(settings: &TriageSettings) -> Triage {
    let model: Option<Arc<dyn TriageModel>> = match settings.provider {
        TriageProvider::Off => None,
        TriageProvider::Fake => Some(Arc::new(FakeTriage)),
        TriageProvider::OpenAi if settings.api_key.is_empty() => {
            tracing::warn!("OPENAI_API_KEY is empty: every intake item will be held for review");
            Some(Arc::new(Unconfigured))
        }
        TriageProvider::OpenAi
        | TriageProvider::Ollama
        | TriageProvider::LlamaCpp
        | TriageProvider::Vllm => Some(Arc::new(OpenAiCompatible::new(
            reqwest::Client::new(),
            &settings.base_url,
            &settings.model,
            &settings.api_key,
        ))),
    };
    tracing::info!(provider = ?settings.provider, model = %settings.model, "triage ready");
    Triage::new(model, settings.timeout, settings.min_confidence)
}

/// Stands in for OpenAI when no key is set, so every item is held with a
/// reason that says what to fix, instead of a bare "answered 401".
struct Unconfigured;

impl TriageModel for Unconfigured {
    fn classify<'a>(&'a self, _: &'a str) -> BoxFuture<'a, Result<Classification, TriageError>> {
        Box::pin(async {
            Err(TriageError::Unavailable(
                "OPENAI_API_KEY is not set".to_string(),
            ))
        })
    }
}
