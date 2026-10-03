//! Helpers for the purchase history tests: filling a trolley the way the
//! bookmarklet does, waiting for the background save, and writing purchases
//! at chosen dates straight into the tables.

use std::time::Duration;

use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use super::trolley::{claim, create, report};
use super::TestApp;

/// Sends every Woolworths choice to the trolley and reports `lines` added or
/// failed, with a reserved window costing `fee`. Returns the handoff id.
pub async fn fill_trolley(app: &TestApp, lines: Value, fee: &str) -> String {
    create(app, "woolworths").await;
    let (_, claimed, _) = claim(app).await;
    let id = claimed["handoff_id"].as_str().unwrap().to_string();
    let (status, body, _) = report(
        app,
        &id,
        json!({
            "lines": lines,
            "delivery": {
                "outcome": "reserved",
                "window_label": "7:00am - 10:00am",
                "window_start": "2026-10-03T07:00:00",
                "window_end": "2026-10-03T10:00:00",
                "fee": fee
            }
        }),
    )
    .await;
    assert!(status.is_success(), "{body}");
    id
}

/// The saved shops, once at least `count` exist. The save runs after the
/// report is answered, so this polls; it fails after five seconds.
pub async fn orders_once_saved(app: &TestApp, count: usize) -> Vec<Value> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let (_, body) = app.get("/api/purchase-orders").await;
        let orders = body.as_array().cloned().unwrap_or_default();
        if orders.len() >= count {
            return orders;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{count} shop(s) were not saved within five seconds: {body}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// One saved purchase, written straight into the tables.
pub struct Bought<'a> {
    pub item: &'a str,
    pub store: &'a str,
    pub product_id: &'a str,
    pub total: &'a str,
    pub delivery_share: &'a str,
    pub category: &'a str,
    /// RFC 3339, e.g. `2026-09-30T13:30:00Z`.
    pub at: &'a str,
}

/// Writes `bought` as a one-product shop.
pub async fn insert_bought(pool: &PgPool, bought: &Bought<'_>) {
    let order_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO purchase_orders (id, store, items_total, delivery_fee, recorded_by, bought_at)
         VALUES ($1, $2, $3::numeric, $4::numeric, 'dev-user', $5::timestamptz)",
    )
    .bind(order_id)
    .bind(bought.store)
    .bind(bought.total)
    .bind(bought.delivery_share)
    .bind(bought.at)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO purchases (id, order_id, item_name, item_key, item_status_before, store,
                                product_id, product_name, quantity, unit_price, total_price,
                                delivery_fee_share, category, category_source, bought_at)
         VALUES ($1, $2, $3, lower($3), 'committed', $4, $5, $5, 1, $6::numeric, $6::numeric,
                 $7::numeric, $8, 'name', $9::timestamptz)",
    )
    .bind(Uuid::new_v4())
    .bind(order_id)
    .bind(bought.item)
    .bind(bought.store)
    .bind(bought.product_id)
    .bind(bought.total)
    .bind(bought.delivery_share)
    .bind(bought.category)
    .bind(bought.at)
    .execute(pool)
    .await
    .unwrap();
}
