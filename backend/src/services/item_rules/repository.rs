//! Every SQL statement item rules need.
//!
//! Only `item_rules` is touched here. Functions take an executor, so the list
//! and the intake queue can read rules inside the transaction that adds the
//! item they apply to. Every statement is a literal with bind parameters.

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::ItemRule;

/// Every rule, in the order they were made.
pub async fn list_rules<'e, E>(executor: E) -> Result<Vec<ItemRule>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemRule>(
        "SELECT id, triggers, filter_terms, apply_to_manual, created_at, updated_at
         FROM item_rules
         ORDER BY created_at, id",
    )
    .fetch_all(executor)
    .await?)
}

/// One rule by id.
pub async fn find_rule<'e, E>(executor: E, rule_id: Uuid) -> Result<Option<ItemRule>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemRule>(
        "SELECT id, triggers, filter_terms, apply_to_manual, created_at, updated_at
         FROM item_rules
         WHERE id = $1",
    )
    .bind(rule_id)
    .fetch_optional(executor)
    .await?)
}

/// Stores a new rule.
pub async fn insert_rule<'e, E>(
    executor: E,
    triggers: &[String],
    filter_terms: &[String],
    apply_to_manual: bool,
) -> Result<ItemRule, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemRule>(
        "INSERT INTO item_rules (id, triggers, filter_terms, apply_to_manual)
         VALUES ($1, $2, $3, $4)
         RETURNING id, triggers, filter_terms, apply_to_manual, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(triggers)
    .bind(filter_terms)
    .bind(apply_to_manual)
    .fetch_one(executor)
    .await?)
}

/// Writes every editable field at once. `None` means no rule matched.
///
/// The caller resolves "leave this as it was" before calling, so each bind is
/// the value the row should end up with.
pub async fn update_rule<'e, E>(
    executor: E,
    rule_id: Uuid,
    triggers: &[String],
    filter_terms: &[String],
    apply_to_manual: bool,
) -> Result<Option<ItemRule>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, ItemRule>(
        "UPDATE item_rules
         SET triggers = $2, filter_terms = $3, apply_to_manual = $4, updated_at = now()
         WHERE id = $1
         RETURNING id, triggers, filter_terms, apply_to_manual, created_at, updated_at",
    )
    .bind(rule_id)
    .bind(triggers)
    .bind(filter_terms)
    .bind(apply_to_manual)
    .fetch_optional(executor)
    .await?)
}

/// Removes a rule. `false` means no row matched.
pub async fn delete_rule<'e, E>(executor: E, rule_id: Uuid) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query("DELETE FROM item_rules WHERE id = $1")
        .bind(rule_id)
        .execute(executor)
        .await?;
    Ok(result.rows_affected() > 0)
}
