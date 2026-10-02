//! Settings for the LLM triage step (`docs/features/FEATURE_TRIAGE.md`).
//!
//! `INTAKE_LLM_PROVIDER` picks the classifier. OpenAI and the three local
//! model servers (Ollama, llama.cpp's `llama-server`, vLLM) all speak the
//! OpenAI chat-completions protocol, so they share one client and differ only
//! in the defaults below.

use std::fmt;
use std::time::Duration;

use super::{optional, parse_or};
use crate::error::ConfigError;

/// Which classifier screens intake requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriageProvider {
    /// OpenAI's API, in the cloud. Costs money per item.
    OpenAi,
    /// A local Ollama server, through its OpenAI-compatible endpoint. The
    /// default: free, and item names never leave the home server.
    Ollama,
    /// llama.cpp's `llama-server`, serving one GGUF model.
    LlamaCpp,
    /// A vLLM server (`vllm serve`). Needs a GPU.
    Vllm,
    /// A small keyword list; nothing leaves the machine. For development.
    Fake,
    /// No triage: every request goes straight to Pending Requests.
    Off,
}

/// How the backend reaches the triage classifier.
#[derive(Clone)]
pub struct TriageSettings {
    /// `INTAKE_LLM_PROVIDER=ollama|llamacpp|vllm|openai|fake|off`. Defaults to
    /// `ollama`.
    pub provider: TriageProvider,
    /// The OpenAI-compatible API root, ending before `/chat/completions`.
    pub base_url: String,
    /// The model name the provider is asked for.
    pub model: String,
    /// `OPENAI_API_KEY` for OpenAI; `INTAKE_LLM_API_KEY` for a local server
    /// started with `--api-key` (empty otherwise). Never logged.
    pub api_key: String,
    /// How long one classification may take before the item is held.
    pub timeout: Duration,
    /// Answers below this confidence are held for a person to look at.
    pub min_confidence: f32,
}

impl TriageSettings {
    /// Reads the triage settings from the environment.
    pub(super) fn from_env() -> Result<Self, ConfigError> {
        let provider = parse_provider(optional("INTAKE_LLM_PROVIDER").as_deref())?;
        let min_confidence: f32 = parse_or("INTAKE_LLM_MIN_CONFIDENCE", 0.7)?;
        if !(0.0..=1.0).contains(&min_confidence) {
            return Err(ConfigError::Invalid {
                key: "INTAKE_LLM_MIN_CONFIDENCE".to_string(),
            });
        }
        Ok(Self {
            provider,
            base_url: optional("INTAKE_LLM_BASE_URL")
                .unwrap_or_else(|| default_base_url(provider).to_string()),
            model: optional("INTAKE_LLM_MODEL")
                .unwrap_or_else(|| default_model(provider).to_string()),
            api_key: optional(api_key_variable(provider)).unwrap_or_default(),
            timeout: Duration::from_secs(parse_or("INTAKE_LLM_TIMEOUT_SECONDS", 20)?),
            min_confidence,
        })
    }
}

/// Hand-written so the API key can never reach a log line through `{:?}`.
impl fmt::Debug for TriageSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TriageSettings")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field(
                "api_key",
                &if self.api_key.is_empty() {
                    "<empty>"
                } else {
                    "<set>"
                },
            )
            .field("timeout", &self.timeout)
            .field("min_confidence", &self.min_confidence)
            .finish()
    }
}

/// Reads `INTAKE_LLM_PROVIDER`; unset means `ollama`. The household chose a
/// local model over OpenAI: it is free and keeps item names at home.
/// `llama.cpp`, `llama-cpp` and `llamacpp` are the same server.
fn parse_provider(raw: Option<&str>) -> Result<TriageProvider, ConfigError> {
    match raw
        .map(|value| {
            value
                .trim()
                .to_ascii_lowercase()
                .replace(['-', '.', '_'], "")
        })
        .as_deref()
    {
        None | Some("ollama") => Ok(TriageProvider::Ollama),
        Some("openai") => Ok(TriageProvider::OpenAi),
        Some("llamacpp") => Ok(TriageProvider::LlamaCpp),
        Some("vllm") => Ok(TriageProvider::Vllm),
        Some("fake") => Ok(TriageProvider::Fake),
        Some("off") => Ok(TriageProvider::Off),
        Some(_) => Err(ConfigError::Invalid {
            key: "INTAKE_LLM_PROVIDER".to_string(),
        }),
    }
}

/// Which variable holds the provider's key: OpenAI's own, or the one a local
/// server was started with.
fn api_key_variable(provider: TriageProvider) -> &'static str {
    match provider {
        TriageProvider::OpenAi => "OPENAI_API_KEY",
        _ => "INTAKE_LLM_API_KEY",
    }
}

/// Where each provider listens unless `INTAKE_LLM_BASE_URL` says otherwise:
/// each server's own default port, on this machine. From inside Docker, use
/// the Compose service name instead (`http://ollama:11434/v1`).
fn default_base_url(provider: TriageProvider) -> &'static str {
    match provider {
        TriageProvider::Ollama => "http://localhost:11434/v1",
        TriageProvider::LlamaCpp => "http://localhost:8080/v1",
        TriageProvider::Vllm => "http://localhost:8000/v1",
        TriageProvider::OpenAi | TriageProvider::Fake | TriageProvider::Off => {
            "https://api.openai.com/v1"
        }
    }
}

/// The model each provider is asked for unless `INTAKE_LLM_MODEL` names one.
/// llama.cpp serves whatever model it was started with and ignores the name;
/// vLLM needs the exact name it serves.
fn default_model(provider: TriageProvider) -> &'static str {
    match provider {
        TriageProvider::Ollama => "llama3.2",
        TriageProvider::LlamaCpp => "local",
        TriageProvider::Vllm => "Qwen/Qwen2.5-1.5B-Instruct",
        TriageProvider::OpenAi | TriageProvider::Fake | TriageProvider::Off => "gpt-4o-mini",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_provider_is_the_local_ollama() {
        assert_eq!(parse_provider(None).unwrap(), TriageProvider::Ollama);
    }

    #[test]
    fn local_servers_accept_their_usual_spellings() {
        for spelling in [
            "llamacpp",
            "llama.cpp",
            "llama-cpp",
            "llama_cpp",
            "LlamaCpp",
        ] {
            assert_eq!(
                parse_provider(Some(spelling)).unwrap(),
                TriageProvider::LlamaCpp
            );
        }
        assert_eq!(parse_provider(Some("vLLM")).unwrap(), TriageProvider::Vllm);
        assert_eq!(
            parse_provider(Some("openai")).unwrap(),
            TriageProvider::OpenAi
        );
    }

    #[test]
    fn each_local_server_defaults_to_its_own_port_and_key_variable() {
        assert_eq!(
            default_base_url(TriageProvider::LlamaCpp),
            "http://localhost:8080/v1"
        );
        assert_eq!(
            default_base_url(TriageProvider::Vllm),
            "http://localhost:8000/v1"
        );
        assert_eq!(api_key_variable(TriageProvider::Vllm), "INTAKE_LLM_API_KEY");
        assert_eq!(api_key_variable(TriageProvider::OpenAi), "OPENAI_API_KEY");
    }

    #[test]
    fn provider_names_ignore_case_and_spaces() {
        assert_eq!(
            parse_provider(Some(" Ollama ")).unwrap(),
            TriageProvider::Ollama
        );
        assert_eq!(parse_provider(Some("FAKE")).unwrap(), TriageProvider::Fake);
        assert_eq!(parse_provider(Some("off")).unwrap(), TriageProvider::Off);
    }

    #[test]
    fn an_unknown_provider_is_an_error_not_a_silent_default() {
        assert!(parse_provider(Some("opneai")).is_err());
    }

    #[test]
    fn ollama_has_its_own_local_defaults() {
        assert_eq!(
            default_base_url(TriageProvider::Ollama),
            "http://localhost:11434/v1"
        );
        assert_eq!(default_model(TriageProvider::OpenAi), "gpt-4o-mini");
    }

    #[test]
    fn debug_output_never_shows_the_key() {
        let settings = TriageSettings {
            provider: TriageProvider::OpenAi,
            base_url: String::new(),
            model: String::new(),
            api_key: "sk-very-secret".to_string(),
            timeout: Duration::from_secs(1),
            min_confidence: 0.7,
        };
        let printed = format!("{settings:?}");
        assert!(!printed.contains("sk-very-secret"));
        assert!(printed.contains("<set>"));
    }
}
