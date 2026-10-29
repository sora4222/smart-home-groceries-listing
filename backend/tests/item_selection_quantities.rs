//! Buying several products for one item, through the real router.
//!
//! An item wanting more than one may keep up to that many products at a
//! store. Each product has a count, and the counts add up to the item's
//! quantity. The stores are the fake catalogue.

mod common;

use axum::http::StatusCode;
use common::trolley::{chosen_item, create};
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

fn uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/selection")
}

async fn choose(app: &TestApp, item_id: &str, product_id: &str) -> (StatusCode, Value) {
    app.put_json(
        &uri(item_id),
        &json!({ "store": "woolworths", "product_id": product_id }),
    )
    .await
}

async fn set_count(app: &TestApp, item_id: &str, product_id: &str, quantity: i64) -> StatusCode {
    let (status, _) = app
        .put_json(
            &format!("{}/quantity", uri(item_id)),
            &json!({ "store": "woolworths", "product_id": product_id, "quantity": quantity }),
        )
        .await;
    status
}

/// The item's products the order buys, as (product id, count).
async fn bought(app: &TestApp) -> Vec<(String, i64)> {
    let (status, body) = app.get("/api/item-selections").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let item = &body[0];
    std::iter::once(item)
        .chain(item["bought_with"].as_array().unwrap())
        .map(|row| {
            (
                row["product_id"].as_str().unwrap().to_string(),
                row["quantity"].as_i64().unwrap(),
            )
        })
        .collect()
}

fn pair(product_id: &str, count: i64) -> (String, i64) {
    (product_id.to_string(), count)
}

#[sqlx::test]
async fn a_second_product_takes_one_from_the_first(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 3, "woolworths", "w-milk-2l").await;

    let (status, body) = choose(&app, &milk, "w-milk-3l").await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        bought(&app).await,
        vec![pair("w-milk-2l", 2), pair("w-milk-3l", 1)]
    );
}

#[sqlx::test]
async fn an_item_wanting_one_still_swaps_its_product(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;

    choose(&app, &milk, "w-milk-3l").await;

    assert_eq!(bought(&app).await, vec![pair("w-milk-3l", 1)]);
}

#[sqlx::test]
async fn choosing_a_chosen_product_again_keeps_the_counts(pool: PgPool) {
    // The fake catalogue has two Woolworths milks, so the "no room" 409 is
    // covered by the allocation unit tests.
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 3, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "w-milk-3l").await;

    let (status, body) = choose(&app, &milk, "w-milk-3l").await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        bought(&app).await,
        vec![pair("w-milk-2l", 2), pair("w-milk-3l", 1)]
    );
}

#[sqlx::test]
async fn setting_one_count_moves_the_rest_to_the_others(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 4, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "w-milk-3l").await;

    assert_eq!(
        set_count(&app, &milk, "w-milk-3l", 3).await,
        StatusCode::NO_CONTENT
    );

    assert_eq!(
        bought(&app).await,
        vec![pair("w-milk-2l", 1), pair("w-milk-3l", 3)]
    );
}

#[sqlx::test]
async fn a_count_with_no_room_for_the_others_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 3, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "w-milk-3l").await;

    assert_eq!(
        set_count(&app, &milk, "w-milk-3l", 3).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test]
async fn clearing_one_product_gives_its_count_back(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 3, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "w-milk-3l").await;

    let (status, _) = app
        .delete(&format!(
            "{}?store=woolworths&product_id=w-milk-2l",
            uri(&milk)
        ))
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(bought(&app).await, vec![pair("w-milk-3l", 3)]);
}

#[sqlx::test]
async fn raising_the_quantity_goes_to_the_first_product(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "w-milk-3l").await;

    app.patch_json(
        &format!("/api/grocery-items/{milk}"),
        &json!({ "quantity": 5 }),
    )
    .await;

    let counts = bought(&app).await;
    assert_eq!(counts.iter().map(|(_, n)| n).sum::<i64>(), 5, "{counts:?}");
}

#[sqlx::test]
async fn the_trolley_gets_one_line_per_product(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 3, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "w-milk-3l").await;

    let (status, body) = create(&app, "woolworths").await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    let lines: Vec<_> = body["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|line| {
            (
                line["product_id"].as_str().unwrap().to_string(),
                line["quantity"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(lines, vec![pair("w-milk-2l", 2), pair("w-milk-3l", 1)]);
}
