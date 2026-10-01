//! The Coles client against a mock Coles.
//!
//! Never a real store: `wiremock` stands in for the website, serving
//! fixtures trimmed from real responses.

use std::time::Duration;

use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use grocery_backend::services::stores::coles::ColesClient;
use grocery_backend::services::stores::http::build_client;
use grocery_backend::services::stores::{StoreClient, StoreError};

const BUILD_ID: &str = "20260929.4-53e2baf1c46ef35378fc9f730ad7af8617974443";
const HOME_PAGE: &str = include_str!("fixtures/coles/home.html");

fn search_path(build_id: &str) -> String {
    format!("/_next/data/{build_id}/en/search/products.json")
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/coles/search_soft_drink.json")).unwrap()
}

fn client(server: &MockServer) -> ColesClient {
    let http = build_client(Duration::from_secs(5)).unwrap();
    ColesClient::new(http, &server.uri())
}

async fn mount_home_page(server: &MockServer, body: &str, expected_visits: u64) {
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .expect(expected_visits)
        .mount(server)
        .await;
}

#[tokio::test]
async fn searches_the_data_route_for_the_current_build() {
    let server = MockServer::start().await;
    mount_home_page(&server, HOME_PAGE, 1).await;
    Mock::given(method("GET"))
        .and(path(search_path(BUILD_ID)))
        .and(query_param("q", "soft drink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(2)
        .mount(&server)
        .await;

    let coles = client(&server);
    let products = coles.search("soft drink").await.unwrap();
    // The build id is remembered, not re-read for every search.
    coles.search("soft drink").await.unwrap();

    assert_eq!(products.len(), 4);
    assert_eq!(products[0].product_id, "123011");
}

#[tokio::test]
async fn a_redeploy_reads_the_new_build_id_once() {
    let server = MockServer::start().await;
    let new_id = "20261001.1-feedface";
    // First visit returns the old id, the second the new one.
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(HOME_PAGE))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(HOME_PAGE.replace(BUILD_ID, new_id)),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(search_path(BUILD_ID)))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(search_path(new_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let products = client(&server).search("soft drink").await.unwrap();

    assert_eq!(products.len(), 4);
}

#[tokio::test]
async fn a_home_page_without_a_build_id_is_a_refusal() {
    let server = MockServer::start().await;
    mount_home_page(&server, "<html>Access denied</html>", 1).await;

    let result = client(&server).search("milk").await;

    assert_eq!(result, Err(StoreError::Blocked { status: 200 }));
}

#[tokio::test]
async fn a_refused_search_is_reported_not_retried() {
    let server = MockServer::start().await;
    mount_home_page(&server, HOME_PAGE, 1).await;
    Mock::given(method("GET"))
        .and(path(search_path(BUILD_ID)))
        .respond_with(ResponseTemplate::new(403))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server).search("milk").await;

    assert_eq!(result, Err(StoreError::Blocked { status: 403 }));
}

#[tokio::test]
async fn a_changed_response_shape_is_reported() {
    let server = MockServer::start().await;
    mount_home_page(&server, HOME_PAGE, 1).await;
    Mock::given(method("GET"))
        .and(path(search_path(BUILD_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "props": 1 })))
        .mount(&server)
        .await;

    let result = client(&server).search("milk").await;

    assert!(
        matches!(result, Err(StoreError::UnexpectedResponse(_))),
        "{result:?}"
    );
}
