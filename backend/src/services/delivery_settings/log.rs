//! Settings › Delivery's log events. Nothing here is a secret.

use super::DeliverySettings;

/// A household member saved the delivery settings.
pub fn saved(settings: &DeliverySettings, user_id: &str) {
    let fees: Vec<String> = settings
        .stores
        .iter()
        .map(|rules| {
            format!(
                "{}: fee {:?}, free over {:?}, minimum {:?}",
                rules.store, rules.delivery_fee, rules.free_delivery_over, rules.minimum_order
            )
        })
        .collect();
    tracing::info!(
        fees = ?fees,
        mode = ?settings.mode,
        max_delivery_spend = ?settings.max_delivery_spend,
        user_id,
        "delivery settings saved"
    );
}
