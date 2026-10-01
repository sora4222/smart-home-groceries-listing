//! The Woolworths client against a mock Woolworths.
//!
//! Never a real store: `wiremock` stands in for the website, serving a
//! fixture trimmed from a real response.

use std::time::Duration;

use serde_json::Value;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use grocery_backend::services::stores::http::build_client;
use grocery_backend::services::stores::woolworths::WoolworthsClient;
use grocery_backend::services::stores::{StoreClient, StoreError};

const SEARCH_PATH: &str = "/apis/ui/Search/products";

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/woolworths/search_soft_drink.json")).unwrap()
}

fn client(server: &MockServer) -> WoolworthsClient {
    let http = build_client(Duration::from_secs(5)).unwrap();
    WoolworthsClient::new(http, &server.uri())
}

/// The home page, which sets the cookies the search then carries.
async fn mount_home_page(server: &MockServer, expected_visits: u64) {
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "bm_sz=abc; Path=/")
                .set_body_string("<html></html>"),
        )
        .expect(expected_visits)
        .mount(server)
        .await;
}

#[tokio::test]
async fn searches_with_the_websites_own_request() {
    let server = MockServer::start().await;
    mount_home_page(&server, 1).await;
    Mock::given(method("POST"))
        .and(path(SEARCH_PATH))
        .and(body_partial_json(serde_json::json!({
            "SearchTerm": "soft drink",
            "PageNumber": 1,
            "SortType": "TraderRelevance"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(2)
        .mount(&server)
        .await;

    let woolworths = client(&server);
    let products = woolworths.search("soft drink").await.unwrap();
    // A second search reuses the cookies rather than visiting again.
    woolworths.search("soft drink").await.unwrap();

    assert_eq!(products.len(), 3, "the sponsored product is dropped");
    assert_eq!(products[0].product_id, "7985");
}

#[tokio::test]
async fn carries_the_cookies_the_home_page_set() {
    let server = MockServer::start().await;
    mount_home_page(&server, 1).await;
    Mock::given(method("POST"))
        .and(path(SEARCH_PATH))
        .and(wiremock::matchers::header_regex("cookie", "bm_sz=abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(1)
        .mount(&server)
        .await;

    client(&server).search("soft drink").await.unwrap();
}

#[tokio::test]
async fn a_refusal_is_reported_and_warms_up_again_next_time() {
    let server = MockServer::start().await;
    mount_home_page(&server, 2).await;
    Mock::given(method("POST"))
        .and(path(SEARCH_PATH))
        .respond_with(ResponseTemplate::new(403))
        .expect(2)
        .mount(&server)
        .await;

    let woolworths = client(&server);
    let first = woolworths.search("milk").await;
    let second = woolworths.search("milk").await;

    assert_eq!(first, Err(StoreError::Blocked { status: 403 }));
    assert_eq!(second, Err(StoreError::Blocked { status: 403 }));
}

#[tokio::test]
async fn a_challenge_page_instead_of_json_is_a_refusal() {
    let server = MockServer::start().await;
    mount_home_page(&server, 1).await;
    Mock::given(method("POST"))
        .and(path(SEARCH_PATH))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html; charset=utf-8")
                .set_body_string("<html>Please verify you are human</html>"),
        )
        .mount(&server)
        .await;

    let result = client(&server).search("milk").await;

    assert_eq!(result, Err(StoreError::Blocked { status: 200 }));
}

#[tokio::test]
async fn an_unreachable_store_is_reported() {
    // Nothing listens on port 9 (discard) on the loopback interface.
    let http = build_client(Duration::from_secs(2)).unwrap();
    let woolworths = WoolworthsClient::new(http, "http://127.0.0.1:9");

    let result = woolworths.search("milk").await;

    assert!(
        matches!(result, Err(StoreError::Unreachable(_))),
        "{result:?}"
    );
}
