//! Coles product search through the website's Next.js data route.
//!
//! The search page at `/search/products?q=` is rendered from
//! `/_next/data/<buildId>/en/search/products.json?q=`, which this client
//! reads directly. The build id comes from the home page and is remembered;
//! when Coles deploys, the old id answers 404, so the id is looked up again
//! once and the search repeated once. That is the only repeat — a refusal by
//! bot protection is reported, never retried.

mod build_id;
mod mapping;
mod wire;

use tokio::sync::Mutex;

use super::client::{BoxFuture, StoreClient};
use super::error::StoreError;
use super::http::{read_json, read_text, transport_error};
use super::product::Product;
use super::store::Store;
use wire::SearchPage;

/// Talks to Coles.
pub struct ColesClient {
    http: wreq::Client,
    base_url: String,
    /// The website build id from the last home page visit.
    build_id: Mutex<Option<String>>,
}

impl ColesClient {
    /// A client for `base_url` (the real site, or a mock server in tests).
    pub fn new(http: wreq::Client, base_url: &str) -> Self {
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            build_id: Mutex::new(None),
        }
    }

    /// The remembered build id, fetching it from the home page if needed.
    async fn build_id(&self) -> Result<String, StoreError> {
        let mut cached = self.build_id.lock().await;
        if let Some(id) = cached.as_ref() {
            return Ok(id.clone());
        }
        tracing::debug!(store = "coles", "reading the website build id");
        let response = self
            .http
            .get(format!("{}/", self.base_url))
            .send()
            .await
            .map_err(transport_error)?;
        let html = read_text(response).await?;
        // A page with no build id is a challenge page, not the website.
        let id = build_id::extract(&html).ok_or(StoreError::Blocked { status: 200 })?;
        *cached = Some(id.clone());
        Ok(id)
    }

    /// One attempt at the data route with the current build id. `Ok(None)`
    /// means the id is stale: Coles answered 404 because it has redeployed.
    async fn fetch(&self, query: &str) -> Result<Option<Vec<Product>>, StoreError> {
        let id = self.build_id().await?;
        let response = self
            .http
            .get(format!(
                "{}/_next/data/{id}/en/search/products.json",
                self.base_url
            ))
            .query(&[("q", query)])
            .header("accept", "*/*")
            .header("x-nextjs-data", "1")
            .send()
            .await
            .map_err(transport_error)?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let page: SearchPage = read_json(response).await?;
        Ok(Some(mapping::products(page, &self.base_url)))
    }

    /// Searches, refreshing the build id once if Coles has redeployed.
    async fn search_products(&self, query: &str) -> Result<Vec<Product>, StoreError> {
        if let Some(products) = self.fetch(query).await? {
            return Ok(products);
        }
        tracing::info!(
            store = "coles",
            "website redeployed; reading the new build id"
        );
        *self.build_id.lock().await = None;
        self.fetch(query)
            .await?
            .ok_or_else(|| StoreError::UnexpectedResponse("search route not found".into()))
    }
}

impl StoreClient for ColesClient {
    fn store(&self) -> Store {
        Store::Coles
    }

    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(self.search_products(query))
    }
}
