//! The order planner's log events. Nothing here is a secret.

use super::OrderPlan;

/// The order's options were worked out.
pub fn planned(plan: &OrderPlan) {
    let options: Vec<String> = plan
        .options
        .iter()
        .map(|option| {
            format!(
                "{:?}: total {} (delivery {}), missing {}, acceptable {}",
                option.kind,
                option.total,
                option.delivery_total,
                option.missing.len(),
                option.acceptable()
            )
        })
        .collect();
    tracing::info!(
        mode = ?plan.mode,
        options = ?options,
        recommended = plan.recommended,
        unchosen = plan.unchosen.len(),
        exact = plan.exact,
        "order options planned"
    );
}
