//! The spending analysis through the real router: `GET /api/spending` and
//! `GET /api/spending/item-prices`. Purchases are written straight into the
//! tables at chosen dates.

mod common;

use axum::http::StatusCode;
use common::purchases::{insert_bought, Bought};
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

const MILK_JULY: Bought = Bought {
    item: "Milk",
    store: "coles",
    product_id: "c-milk-3l",
    total: "4.95",
    delivery_share: "2",
    category: "Dairy & eggs",
    at: "2026-07-03T02:00:00Z",
};
const BREAD_JULY: Bought = Bought {
    item: "Bread",
    store: "woolworths",
    product_id: "w-bread",
    total: "4",
    delivery_share: "0",
    category: "Bakery",
    at: "2026-07-20T02:00:00Z",
};
/// 11:30pm on 30 September in Sydney — still September there.
const MILK_SEPTEMBER: Bought = Bought {
    item: "milk",
    store: "woolworths",
    product_id: "w-milk-2l",
    total: "3.10",
    delivery_share: "0",
    category: "Dairy & eggs",
    at: "2026-09-30T13:30:00Z",
};

async fn seeded(pool: PgPool) -> TestApp {
    for bought in [&MILK_JULY, &BREAD_JULY, &MILK_SEPTEMBER] {
        insert_bought(&pool, bought).await;
    }
    TestApp::new(pool)
}

async fn spending(app: &TestApp, query: &str) -> Value {
    let (status, body) = app.get(&format!("/api/spending?{query}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn nothing_bought_yet_is_empty(pool: PgPool) {
    let app = TestApp::new(pool);
    let body = spending(&app, "").await;
    assert_eq!(body["latest_order"], Value::Null);
    assert_eq!(body["total"], "0");
    assert_eq!(body["over_time"], json!([]));
    assert_eq!(body["by_store"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn every_view_with_the_newest_shop_on_top(pool: PgPool) {
    let app = seeded(pool).await;

    let body = spending(&app, "tz=Australia/Sydney&period=month").await;

    assert_eq!(body["latest_order"]["items_total"], "3.10");
    assert_eq!(body["items_total"], "12.05");
    assert_eq!(body["delivery_total"], "2");
    assert_eq!(body["total"], "14.05");
    assert_eq!(
        body["over_time"],
        json!([
            { "start": "2026-07-01", "total": "10.95" },
            { "start": "2026-08-01", "total": "0" },
            { "start": "2026-09-01", "total": "3.10" }
        ])
    );
    assert_eq!(body["by_item"][0]["item_name"], "milk");
    assert_eq!(body["by_item"][0]["times_bought"], 2);
    assert_eq!(body["by_item"][0]["total"], "8.05");
    assert_eq!(body["by_store"][0]["store_name"], "Woolworths");
    assert_eq!(body["by_store"][1]["delivery_total"], "2");
    assert_eq!(body["by_category"][0]["category"], "Dairy & eggs");
    assert_eq!(body["categories"], json!(["Bakery", "Dairy & eggs"]));
}

#[sqlx::test]
async fn days_are_the_households_own(pool: PgPool) {
    let app = seeded(pool).await;

    let sydney = spending(&app, "tz=Australia/Sydney&from=2026-09-01&to=2026-09-30").await;
    let utc = spending(&app, "from=2026-10-01&to=2026-10-31").await;

    assert_eq!(sydney["items_total"], "3.10");
    // In UTC the same shop was still 30 September.
    assert_eq!(utc["items_total"], "0");
}

#[sqlx::test]
async fn filters_narrow_every_view(pool: PgPool) {
    let app = seeded(pool).await;

    let coles = spending(&app, "store=coles").await;
    let milk = spending(&app, "item=%20MILK%20").await;
    let bakery = spending(&app, "category=Bakery").await;

    assert_eq!(coles["items_total"], "4.95");
    assert_eq!(coles["by_store"].as_array().unwrap().len(), 1);
    assert_eq!(milk["items_total"], "8.05");
    assert_eq!(bakery["items_total"], "4");
    // The newest shop and the category list ignore the filters.
    assert_eq!(bakery["latest_order"]["items_total"], "3.10");
    assert_eq!(bakery["categories"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn blank_filters_are_no_filter(pool: PgPool) {
    let app = seeded(pool).await;
    let body = spending(&app, "item=&category=&tz=").await;
    assert_eq!(body["items_total"], "12.05");
}

#[sqlx::test]
async fn weeks_and_quarters(pool: PgPool) {
    let app = seeded(pool).await;

    let quarters = spending(&app, "period=quarter&tz=Australia/Sydney").await;
    let weeks = spending(&app, "period=week&from=2026-07-01&to=2026-07-31").await;

    assert_eq!(quarters["over_time"].as_array().unwrap().len(), 1);
    assert_eq!(quarters["over_time"][0]["start"], "2026-07-01");
    // 3 July is in the week from Monday 29 June; 20 July starts its own week.
    assert_eq!(weeks["over_time"][0]["start"], "2026-06-29");
    assert_eq!(weeks["over_time"].as_array().unwrap().len(), 4);
}

#[sqlx::test]
async fn nonsense_filters_are_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    for query in [
        "tz=Mars/Olympus",
        "from=2026-09-30&to=2026-09-01",
        "store=aldi",
        "period=fortnight",
        "from=yesterday",
    ] {
        let (status, body) = app.get(&format!("/api/spending?{query}")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{query}: {body}");
    }
}

#[sqlx::test]
async fn an_items_price_on_every_shop_oldest_first(pool: PgPool) {
    let app = seeded(pool).await;

    let (status, body) = app.get("/api/spending/item-prices?name=Milk").await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let prices = body.as_array().unwrap();
    assert_eq!(prices.len(), 2);
    assert_eq!(prices[0]["store_name"], "Coles");
    assert_eq!(prices[0]["unit_price"], "4.95");
    assert_eq!(prices[1]["store"], "woolworths");
    assert_eq!(prices[1]["unit_price"], "3.10");
}

#[sqlx::test]
async fn an_item_name_is_needed_for_its_prices(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.get("/api/spending/item-prices").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn spending_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let (status, _) = app.get("/api/spending").await;
    let (prices, _) = app.get("/api/spending/item-prices?name=milk").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(prices, StatusCode::UNAUTHORIZED);
}
