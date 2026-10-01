//! Woolworths product search through the website's own JSON endpoint.
//!
//! The website calls `POST /apis/ui/Search/products`; this client makes the
//! same call. Woolworths sits behind Akamai, whose cookies are set by a
//! normal page visit, so the first search of a client's life visits the home
//! page first ("warming up") and the cookie store carries the result. A
//! refused search clears that, so the next search — never this one — warms
//! up again. Nothing here retries in a loop.

mod mapping;
mod wire;

use tokio::sync::Mutex;

use super::client::{BoxFuture, StoreClient};
use super::error::StoreError;
use super::http::{read_json, read_text, transport_error};
use super::product::Product;
use super::store::Store;
use wire::{SearchRequest, SearchResponse};

/// How many products one search asks for.
const PAGE_SIZE: u32 = 24;

/// Talks to Woolworths.
pub struct WoolworthsClient {
    http: wreq::Client,
    base_url: String,
    /// The home page has been visited and its cookies are in the jar.
    warmed_up: Mutex<bool>,
}

impl WoolworthsClient {
    /// A client for `base_url` (the real site, or a mock server in tests).
    pub fn new(http: wreq::Client, base_url: &str) -> Self {
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            warmed_up: Mutex::new(false),
        }
    }

    /// Visits the home page once, so the search call carries the cookies a
    /// browser would have.
    async fn warm_up(&self) -> Result<(), StoreError> {
        let mut warmed_up = self.warmed_up.lock().await;
        if *warmed_up {
            return Ok(());
        }
        tracing::debug!(store = "woolworths", "visiting the home page for cookies");
        let response = self
            .http
            .get(format!("{}/", self.base_url))
            .send()
            .await
            .map_err(transport_error)?;
        read_text(response).await?;
        *warmed_up = true;
        Ok(())
    }

    /// The search itself, once warmed up.
    async fn search_products(&self, query: &str) -> Result<Vec<Product>, StoreError> {
        self.warm_up().await?;
        let body = SearchRequest {
            filters: [],
            is_special: false,
            location: format!("/shop/search/products?searchTerm={query}"),
            page_number: 1,
            page_size: PAGE_SIZE,
            search_term: query,
            sort_type: "TraderRelevance",
        };
        let response = self
            .http
            .post(format!("{}/apis/ui/Search/products", self.base_url))
            .header("accept", "application/json, text/plain, */*")
            .header("origin", self.base_url.as_str())
            .header("referer", format!("{}/shop/search/products", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(transport_error)?;
        let parsed: SearchResponse = read_json(response).await?;
        Ok(mapping::products(parsed, &self.base_url))
    }
}

impl StoreClient for WoolworthsClient {
    fn store(&self) -> Store {
        Store::Woolworths
    }

    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(async move {
            let result = self.search_products(query).await;
            if let Err(StoreError::Blocked { .. }) = result {
                // Cookies may have expired: warm up again next time.
                *self.warmed_up.lock().await = false;
            }
            result
        })
    }
}
