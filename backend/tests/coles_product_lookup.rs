//! Looking up one Coles product by id, against a mock Coles.
//!
//! Fixtures are trimmed from the live product data route on 2026-10-06.

use std::time::Duration;

use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use grocery_backend::services::stores::coles::ColesClient;
use grocery_backend::services::stores::http::build_client;
use grocery_backend::services::stores::StoreClient;

const BUILD_ID: &str = "20260929.4-53e2baf1c46ef35378fc9f730ad7af8617974443";
const HOME_PAGE: &str = include_str!("fixtures/coles/home.html");

fn page(name: &str) -> String {
    format!("/_next/data/{BUILD_ID}/en/product/{name}.json")
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

async fn coles() -> (MockServer, ColesClient) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(HOME_PAGE))
        .mount(&server)
        .await;
    let http = build_client(Duration::from_secs(5)).unwrap();
    let client = ColesClient::new(http, &server.uri());
    (server, client)
}

#[tokio::test]
async fn follows_the_redirect_to_the_product_page() {
    let (server, coles) = coles().await;
    Mock::given(method("GET"))
        .and(path(page("x-8150288")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json(include_str!("fixtures/coles/product_redirect.json"))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(page("coles-full-cream-milk-3l-8150288")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json(include_str!("fixtures/coles/product_milk.json"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let milk = coles.product("8150288").await.unwrap().unwrap();

    assert_eq!(milk.name, "Full Cream Milk");
    assert_eq!(milk.package_size.as_deref(), Some("3L"));
    assert_eq!(milk.price.unwrap().to_string(), "4.95");
}

#[tokio::test]
async fn an_unknown_product_is_none_not_a_redeploy() {
    let (server, coles) = coles().await;
    Mock::given(method("GET"))
        .and(path(page("x-999999999")))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json(include_str!("fixtures/coles/product_not_found.json"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    assert_eq!(coles.product("999999999").await.unwrap(), None);
}

#[tokio::test]
async fn an_id_that_is_not_a_coles_id_never_reaches_coles() {
    let (server, coles) = coles().await;

    assert_eq!(coles.product("../admin").await.unwrap(), None);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_stale_build_id_is_read_again_once() {
    let (server, coles) = coles().await;
    // Coles answers `{}` for a build that is no longer deployed.
    Mock::given(method("GET"))
        .and(path(page("x-8150288")))
        .respond_with(ResponseTemplate::new(404).set_body_string("{}"))
        .expect(2)
        .mount(&server)
        .await;

    assert!(coles.product("8150288").await.is_err());
}
