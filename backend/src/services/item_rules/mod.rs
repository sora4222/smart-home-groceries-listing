//! Item rules: persistent filter chips attached to grocery item names.
//!
//! [`ItemRuleService`] manages the rules themselves — the settings page's
//! list, add, edit and delete. [`apply`] works out the chips an item gets as it
//! reaches the list, using [`matching`] to decide which rules fit its name.
//! SQL lives in [`repository`]; tidying trigger phrases lives in [`triggers`].
//!
//! A rule is a template, not a link: its chips are copied onto an item when
//! the item reaches the list, so changing or deleting a rule never touches an
//! item already there.

pub mod apply;
pub mod matching;
pub mod repository;
pub mod triggers;

pub use apply::{filter_terms_for, AddedVia};

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::ItemRule;
use crate::models::schemas::{ItemRuleCreate, ItemRuleUpdate};
use crate::services::filter_terms;

/// Reads and writes item rules.
pub struct ItemRuleService<'a> {
    pool: &'a PgPool,
}

impl<'a> ItemRuleService<'a> {
    /// Borrows the pool for the duration of one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Every rule, in the order they were made.
    pub async fn list(&self) -> Result<Vec<ItemRule>, ApiError> {
        repository::list_rules(self.pool).await
    }

    /// Stores a new rule after tidying its triggers and filter terms.
    ///
    /// 422 when either list is empty once tidied — a rule that matches nothing,
    /// or applies nothing, is a mistake rather than a rule.
    pub async fn create(&self, payload: &ItemRuleCreate) -> Result<ItemRule, ApiError> {
        let triggers = required(triggers::clean(&payload.triggers), "triggers")?;
        let terms = required(filter_terms::clean(&payload.filter_terms), "filter_terms")?;

        let rule =
            repository::insert_rule(self.pool, &triggers, &terms, payload.apply_to_manual).await?;
        tracing::info!(
            rule_id = %rule.id,
            triggers = ?rule.triggers,
            filter_terms = ?rule.filter_terms,
            apply_to_manual = rule.apply_to_manual,
            "item rule created"
        );
        Ok(rule)
    }

    /// Applies an edit to a rule. Absent fields are left as they were.
    pub async fn update(
        &self,
        rule_id: Uuid,
        payload: &ItemRuleUpdate,
    ) -> Result<ItemRule, ApiError> {
        let existing = repository::find_rule(self.pool, rule_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(rule_id.to_string()))?;

        let triggers = match payload.triggers.as_deref() {
            Some(sent) => required(triggers::clean(sent), "triggers")?,
            None => existing.triggers,
        };
        let terms = match payload.filter_terms.as_deref() {
            Some(sent) => required(filter_terms::clean(sent), "filter_terms")?,
            None => existing.filter_terms,
        };
        let apply_to_manual = payload.apply_to_manual.unwrap_or(existing.apply_to_manual);

        // A concurrent delete between the read and this write is a 404 too.
        let rule = repository::update_rule(self.pool, rule_id, &triggers, &terms, apply_to_manual)
            .await?
            .ok_or_else(|| ApiError::NotFound(rule_id.to_string()))?;
        tracing::info!(
            rule_id = %rule.id,
            triggers = ?rule.triggers,
            filter_terms = ?rule.filter_terms,
            apply_to_manual = rule.apply_to_manual,
            "item rule updated"
        );
        Ok(rule)
    }

    /// Removes a rule. Chips it already put on items stay where they are.
    pub async fn delete(&self, rule_id: Uuid) -> Result<(), ApiError> {
        if !repository::delete_rule(self.pool, rule_id).await? {
            return Err(ApiError::NotFound(rule_id.to_string()));
        }
        tracing::info!(rule_id = %rule_id, "item rule deleted");
        Ok(())
    }
}

/// Refuses a list that tidied down to nothing.
fn required(values: Vec<String>, field: &str) -> Result<Vec<String>, ApiError> {
    if values.is_empty() {
        return Err(ApiError::UnprocessableEntity(format!(
            "{field}: at least one non-blank entry is required"
        )));
    }
    Ok(values)
}
