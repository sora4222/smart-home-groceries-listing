//! Turning a classifier's answer into a triage status. Pure, no I/O.
//!
//! Triage only decides which queue a person sees the item in. Nothing here
//! accepts an item for the household.

use super::model::{Classification, TriageError};
use crate::models::db::TriageStatus;

/// Longest reason stored, matching the column's `CHECK`.
pub const MAX_REASON_CHARS: usize = 500;

/// Where an intake request stands after triage, ready to store.
#[derive(Debug, Clone, PartialEq)]
pub struct TriageOutcome {
    pub status: TriageStatus,
    pub reason: Option<String>,
    pub confidence: Option<f32>,
}

impl TriageOutcome {
    /// Triage was not run because it is switched off.
    pub fn switched_off() -> Self {
        Self {
            status: TriageStatus::Skipped,
            reason: Some("Triage is switched off.".to_string()),
            confidence: None,
        }
    }
}

/// Decides the status for one classifier answer.
///
/// * a failure → `held`, with the failure as the reason;
/// * an answer below `min_confidence` → `held`, keeping its reason;
/// * otherwise `approved` or `rejected` as the classifier said.
pub fn decide(answer: Result<Classification, TriageError>, min_confidence: f32) -> TriageOutcome {
    let answer = match answer {
        Ok(answer) => answer,
        Err(err) => {
            return TriageOutcome {
                status: TriageStatus::Held,
                reason: Some(tidy_reason(&capitalise(&err.to_string()))),
                confidence: None,
            }
        }
    };
    let confidence = clamp_confidence(answer.confidence);
    let status = if confidence < min_confidence {
        TriageStatus::Held
    } else if answer.is_supermarket_item {
        TriageStatus::Approved
    } else {
        TriageStatus::Rejected
    };
    TriageOutcome {
        status,
        reason: Some(tidy_reason(&answer.reason)),
        confidence: Some(confidence),
    }
}

/// Keeps a confidence inside 0..=1; a NaN counts as no confidence at all.
fn clamp_confidence(confidence: f32) -> f32 {
    if confidence.is_nan() {
        0.0
    } else {
        confidence.clamp(0.0, 1.0)
    }
}

/// Collapses whitespace and cuts the reason to the column's limit.
fn tidy_reason(reason: &str) -> String {
    let collapsed = reason.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return "No reason given.".to_string();
    }
    collapsed.chars().take(MAX_REASON_CHARS).collect()
}

/// Upper-cases the first letter, so an error reads as a sentence.
fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(is_supermarket_item: bool, confidence: f32) -> Classification {
        Classification {
            is_supermarket_item,
            confidence,
            reason: "Milk is sold at supermarkets.".to_string(),
        }
    }

    #[test]
    fn a_confident_yes_is_approved() {
        let outcome = decide(Ok(answer(true, 0.95)), 0.7);
        assert_eq!(outcome.status, TriageStatus::Approved);
        assert_eq!(outcome.confidence, Some(0.95));
    }

    #[test]
    fn a_confident_no_is_rejected() {
        assert_eq!(
            decide(Ok(answer(false, 0.9)), 0.7).status,
            TriageStatus::Rejected
        );
    }

    #[test]
    fn low_confidence_is_held_either_way() {
        assert_eq!(
            decide(Ok(answer(true, 0.5)), 0.7).status,
            TriageStatus::Held
        );
        assert_eq!(
            decide(Ok(answer(false, 0.5)), 0.7).status,
            TriageStatus::Held
        );
    }

    #[test]
    fn exactly_the_threshold_is_confident_enough() {
        assert_eq!(
            decide(Ok(answer(true, 0.7)), 0.7).status,
            TriageStatus::Approved
        );
    }

    #[test]
    fn a_failure_is_held_with_the_failure_as_the_reason() {
        let outcome = decide(Err(TriageError::BadReply), 0.7);
        assert_eq!(outcome.status, TriageStatus::Held);
        assert_eq!(outcome.confidence, None);
        assert_eq!(
            outcome.reason.as_deref(),
            Some("The checker gave an answer that could not be read")
        );
    }

    #[test]
    fn out_of_range_confidence_is_clamped_and_nan_is_held() {
        assert_eq!(decide(Ok(answer(true, 3.0)), 0.7).confidence, Some(1.0));
        assert_eq!(
            decide(Ok(answer(true, f32::NAN)), 0.7).status,
            TriageStatus::Held
        );
    }

    #[test]
    fn reasons_are_tidied_and_cut_to_the_column_limit() {
        let mut long = answer(true, 0.9);
        long.reason = format!("  many   spaces {}", "x".repeat(900));
        let reason = decide(Ok(long), 0.7).reason.unwrap();
        assert!(reason.starts_with("many spaces x"));
        assert_eq!(reason.chars().count(), MAX_REASON_CHARS);

        let mut blank = answer(true, 0.9);
        blank.reason = "   ".to_string();
        assert_eq!(
            decide(Ok(blank), 0.7).reason.as_deref(),
            Some("No reason given.")
        );
    }
}
