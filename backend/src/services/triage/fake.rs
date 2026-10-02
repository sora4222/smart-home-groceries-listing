//! A classifier that never leaves the machine, for development and tests.
//!
//! `INTAKE_LLM_PROVIDER=fake`. It says "not a supermarket item" when the text
//! names a service from a short list, and "yes" to everything else. It is not
//! meant to be clever, only predictable.

use super::model::{Classification, TriageError, TriageModel};
use crate::services::stores::client::BoxFuture;

/// Words that mark a service rather than something on a shelf.
const SERVICE_WORDS: [&str; 8] = [
    "service",
    "repair",
    "appointment",
    "haircut",
    "plumber",
    "dentist",
    "rego",
    "mechanic",
];

/// The keyword classifier.
#[derive(Debug, Default, Clone, Copy)]
pub struct FakeTriage;

impl TriageModel for FakeTriage {
    fn classify<'a>(
        &'a self,
        item_text: &'a str,
    ) -> BoxFuture<'a, Result<Classification, TriageError>> {
        Box::pin(async move { Ok(classify_by_keyword(item_text)) })
    }
}

/// The fake's whole rule: a service word means no.
fn classify_by_keyword(item_text: &str) -> Classification {
    let lowered = item_text.to_lowercase();
    let is_service = lowered
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| SERVICE_WORDS.contains(&word));
    if is_service {
        Classification {
            is_supermarket_item: false,
            confidence: 0.9,
            reason: "Sounds like a service, not something a supermarket sells.".to_string(),
        }
    } else {
        Classification {
            is_supermarket_item: true,
            confidence: 0.9,
            reason: "Sounds like something a supermarket sells.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn groceries_are_supermarket_items() {
        let answer = FakeTriage.classify("two oat milk").await.unwrap();
        assert!(answer.is_supermarket_item);
    }

    #[tokio::test]
    async fn a_service_word_is_not() {
        let answer = FakeTriage.classify("Car Service on Friday").await.unwrap();
        assert!(!answer.is_supermarket_item);
    }

    #[tokio::test]
    async fn only_whole_words_count() {
        // "serviceberry" is a fruit, not a service.
        let answer = FakeTriage.classify("serviceberry jam").await.unwrap();
        assert!(answer.is_supermarket_item);
    }
}
