//! Looking up one Woolworths product by stockcode, against a mock
//! Woolworths.
//!
//! Fixtures are trimmed from the live `product/detail` call on 2026-10-06.

use std::time::Duration;

use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use grocery_backend::services::stores::http::build_client;
use grocery_backend::services::stores::woolworths::WoolworthsClient;
use grocery_backend::services::stores::{StoreClient, StoreError};

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

async fn woolworths() -> (MockServer, WoolworthsClient) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html></html>"))
        .mount(&server)
        .await;
    let http = build_client(Duration::from_secs(5)).unwrap();
    let client = WoolworthsClient::new(http, &server.uri());
    (server, client)
}

#[tokio::test]
async fn reads_the_product_page_data() {
    let (server, woolworths) = woolworths().await;
    Mock::given(method("GET"))
        .and(path("/apis/ui/product/detail/722"))
        .and(query_param("isMobile", "false"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json(include_str!("fixtures/woolworths/product_722.json"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let paper = woolworths.product("722").await.unwrap().unwrap();

    assert_eq!(paper.brand.as_deref(), Some("Kleenex"));
    assert_eq!(paper.price.unwrap().to_string(), "12");
    assert!(paper.on_special);
}

#[tokio::test]
async fn an_unknown_stockcode_is_none() {
    let (server, woolworths) = woolworths().await;
    Mock::given(method("GET"))
        .and(path("/apis/ui/product/detail/999999999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json(include_str!(
            "fixtures/woolworths/product_not_found.json"
        ))))
        .mount(&server)
        .await;

    assert_eq!(woolworths.product("999999999").await.unwrap(), None);
}

#[tokio::test]
async fn a_stockcode_that_is_not_digits_never_reaches_woolworths() {
    let (server, woolworths) = woolworths().await;

    assert_eq!(woolworths.product("722/../x").await.unwrap(), None);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_refusal_is_reported() {
    let (server, woolworths) = woolworths().await;
    Mock::given(method("GET"))
        .and(path("/apis/ui/product/detail/722"))
        .respond_with(ResponseTemplate::new(403))
        .expect(1)
        .mount(&server)
        .await;

    assert_eq!(
        woolworths.product("722").await,
        Err(StoreError::Blocked { status: 403 })
    );
}
