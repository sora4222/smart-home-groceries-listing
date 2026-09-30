//! Item rules' log events, one function per thing that happened.
//!
//! Matching has its own events in [`super::apply`], next to the decision.

use uuid::Uuid;

use crate::models::db::ItemRule;

/// A rule was created or changed; `change` says which.
pub fn saved(rule: &ItemRule, change: &'static str) {
    tracing::info!(
        rule_id = %rule.id,
        triggers = ?rule.triggers,
        filter_terms = ?rule.filter_terms,
        apply_to_manual = rule.apply_to_manual,
        change,
        "item rule saved"
    );
}

/// A rule was deleted.
pub fn deleted(rule_id: Uuid) {
    tracing::info!(rule_id = %rule_id, "item rule deleted");
}
