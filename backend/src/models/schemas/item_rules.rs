//! Bodies for managing item rules on the settings page.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use super::common::validate_filter_terms;
use crate::models::db::ItemRule;

/// How many trigger phrases one rule may carry, matching the `triggers` CHECK.
pub const MAX_TRIGGERS: usize = 10;
/// Longest single trigger phrase, matching the `triggers` CHECK.
pub const MAX_TRIGGER_LEN: usize = 100;

/// An item rule as the web app sees it.
#[derive(Debug, Serialize)]
pub struct ItemRuleResponse {
    pub id: Uuid,
    pub triggers: Vec<String>,
    pub filter_terms: Vec<String>,
    pub apply_to_manual: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ItemRule> for ItemRuleResponse {
    fn from(row: ItemRule) -> Self {
        Self {
            id: row.id,
            triggers: row.triggers,
            filter_terms: row.filter_terms,
            apply_to_manual: row.apply_to_manual,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// A new rule from the settings page's add form.
///
/// Both lists must name at least one entry once tidied; the service checks
/// that after trimming, since `["  "]` passes these length checks.
#[derive(Debug, Deserialize, Validate)]
pub struct ItemRuleCreate {
    #[validate(length(min = 1), custom(function = "validate_triggers"))]
    pub triggers: Vec<String>,
    #[validate(length(min = 1), custom(function = "validate_filter_terms"))]
    pub filter_terms: Vec<String>,
    #[serde(default)]
    pub apply_to_manual: bool,
}

/// Changes to an existing rule. Absent fields are left as they were; a list
/// that is present replaces the stored one wholesale.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct ItemRuleUpdate {
    #[validate(length(min = 1), custom(function = "validate_triggers"))]
    pub triggers: Option<Vec<String>>,
    #[validate(length(min = 1), custom(function = "validate_filter_terms"))]
    pub filter_terms: Option<Vec<String>>,
    pub apply_to_manual: Option<bool>,
}

/// Rejects a trigger list the `triggers` CHECK constraint would also reject.
fn validate_triggers(triggers: &[String]) -> Result<(), ValidationError> {
    if triggers.len() > MAX_TRIGGERS {
        return Err(ValidationError::new("too_many_triggers"));
    }
    if triggers
        .iter()
        .any(|trigger| trigger.trim().chars().count() > MAX_TRIGGER_LEN)
    {
        return Err(ValidationError::new("trigger_too_long"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_triggers;

    #[test]
    fn ten_triggers_are_accepted() {
        let triggers: Vec<String> = (0..10).map(|n| format!("item {n}")).collect();
        assert!(validate_triggers(&triggers).is_ok());
    }

    #[test]
    fn eleven_triggers_are_too_many() {
        let triggers: Vec<String> = (0..11).map(|n| format!("item {n}")).collect();
        assert!(validate_triggers(&triggers).is_err());
    }

    #[test]
    fn a_trigger_over_a_hundred_characters_is_too_long() {
        assert!(validate_triggers(&["x".repeat(101)]).is_err());
    }
}
