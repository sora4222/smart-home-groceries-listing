//! Running one classifier on one item, with a time limit.

use std::sync::Arc;
use std::time::Duration;

use super::model::{TriageError, TriageModel};
use super::verdict::{decide, TriageOutcome};

/// The configured classifier, or none when triage is switched off.
pub struct Triage {
    model: Option<Arc<dyn TriageModel>>,
    timeout: Duration,
    min_confidence: f32,
}

impl Triage {
    /// `model: None` switches triage off.
    pub fn new(
        model: Option<Arc<dyn TriageModel>>,
        timeout: Duration,
        min_confidence: f32,
    ) -> Self {
        Self {
            model,
            timeout,
            min_confidence,
        }
    }

    /// Whether new requests should wait for the classifier.
    pub fn is_enabled(&self) -> bool {
        self.model.is_some()
    }

    /// Classifies one item. Never fails: a classifier that errors or runs
    /// past the time limit leaves the item `held` for a person.
    pub async fn assess(&self, item_text: &str) -> TriageOutcome {
        let Some(model) = &self.model else {
            return TriageOutcome::switched_off();
        };
        let answer = tokio::time::timeout(self.timeout, model.classify(item_text))
            .await
            .unwrap_or_else(|_| Err(TriageError::Unavailable("timed out".to_string())));
        decide(answer, self.min_confidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::db::TriageStatus;
    use crate::services::stores::client::BoxFuture;
    use crate::services::triage::model::Classification;
    use crate::services::triage::FakeTriage;

    /// A classifier that never answers.
    struct Silent;

    impl TriageModel for Silent {
        fn classify<'a>(
            &'a self,
            _: &'a str,
        ) -> BoxFuture<'a, Result<Classification, TriageError>> {
            Box::pin(std::future::pending())
        }
    }

    #[tokio::test]
    async fn switched_off_skips() {
        let triage = Triage::new(None, Duration::from_secs(1), 0.7);
        assert!(!triage.is_enabled());
        assert_eq!(triage.assess("milk").await.status, TriageStatus::Skipped);
    }

    #[tokio::test]
    async fn a_classifier_past_the_time_limit_holds_the_item() {
        let triage = Triage::new(Some(Arc::new(Silent)), Duration::from_millis(10), 0.7);
        let outcome = triage.assess("milk").await;
        assert_eq!(outcome.status, TriageStatus::Held);
        assert_eq!(
            outcome.reason.as_deref(),
            Some("The checker could not be reached (timed out)")
        );
    }

    #[tokio::test]
    async fn a_confident_answer_passes_through() {
        let triage = Triage::new(Some(Arc::new(FakeTriage)), Duration::from_secs(1), 0.7);
        assert_eq!(triage.assess("milk").await.status, TriageStatus::Approved);
        assert_eq!(
            triage.assess("car service").await.status,
            TriageStatus::Rejected
        );
    }
}
