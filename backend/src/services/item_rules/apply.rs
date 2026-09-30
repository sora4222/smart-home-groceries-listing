//! Working out the chips an item gets from the rules as it reaches the list.
//!
//! Called by the grocery list (items typed into the web app) and the intake
//! queue (accepted voice items) inside the transaction that inserts the item,
//! so a rule edited at the same moment is either seen whole or not at all.

use sqlx::PgExecutor;

use super::{matching, repository};
use crate::error::ApiError;
use crate::services::filter_terms;

/// How an item is reaching the list, which decides the rules that apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddedVia {
    /// Accepted from the intake queue: every rule applies.
    Voice,
    /// Typed into the web app: only rules with `apply_to_manual` apply.
    Manual,
}

/// The filter terms the matching rules give an item called `item_name`, in
/// rule order, tidied and without repeats. Empty when no rule matches.
pub async fn filter_terms_for<'e, E>(
    executor: E,
    item_name: &str,
    via: AddedVia,
) -> Result<Vec<String>, ApiError>
where
    E: PgExecutor<'e>,
{
    let rules = match via {
        AddedVia::Voice => repository::list_rules(executor).await?,
        AddedVia::Manual => repository::list_manual_rules(executor).await?,
    };
    let matched = matching::matching_rules(item_name, &rules);
    if matched.is_empty() {
        tracing::debug!(item = item_name, ?via, "no item rule matched");
        return Ok(Vec::new());
    }

    let terms: Vec<String> = matched
        .iter()
        .flat_map(|rule| rule.filter_terms.iter().cloned())
        .collect();
    let terms = filter_terms::clean(&terms);
    tracing::info!(
        item = item_name,
        ?via,
        rule_ids = ?matched.iter().map(|rule| rule.id).collect::<Vec<_>>(),
        filter_terms = ?terms,
        "item rules matched"
    );
    Ok(terms)
}
