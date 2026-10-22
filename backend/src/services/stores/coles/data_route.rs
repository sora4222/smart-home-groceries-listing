//! Reading a Coles page's data through its Next.js data route.
//!
//! Every Coles page is rendered from
//! `/_next/data/<buildId>/en/<page>.json`, which this module reads
//! directly. The build id comes from the home page and is remembered.
//!
//! Coles answers 404 in two ways, told apart by the body (both captured
//! live on 2026-10-06):
//! - `{}` — the build id is stale because Coles redeployed. The id is read
//!   again once and the request repeated once.
//! - `{"notFound":true}` — the page does not exist (an unknown product).
//!
//! That one repeat is the only one: a refusal by bot protection is
//! reported, never retried.

use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

use super::build_id;
use crate::services::stores::error::StoreError;
use crate::services::stores::http::{read_json, read_text, transport_error};

/// What a data route answered.
#[derive(Debug)]
pub enum DataPage<T> {
    /// The page's data.
    Found(T),
    /// Coles has no such page.
    NotFound,
}

/// A data route reader for one Coles site.
pub struct DataRoute {
    http: wreq::Client,
    base_url: String,
    /// The website build id from the last home page visit.
    build_id: Mutex<Option<String>>,
}

/// A 404 that names a missing page, as opposed to a stale build id.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotFoundBody {
    #[serde(default)]
    not_found: bool,
}

impl DataRoute {
    /// A reader for `base_url` (the real site, or a mock server in tests).
    pub fn new(http: wreq::Client, base_url: &str) -> Self {
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            build_id: Mutex::new(None),
        }
    }

    /// The site's address, without a trailing slash.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Reads `page` (e.g. `search/products`) with `query`, refreshing the
    /// build id once if Coles has redeployed.
    pub async fn get<T: DeserializeOwned>(
        &self,
        page: &str,
        query: &[(&str, &str)],
    ) -> Result<DataPage<T>, StoreError> {
        if let Some(found) = self.fetch(page, query).await? {
            return Ok(found);
        }
        tracing::info!(
            store = "coles",
            "website redeployed; reading the new build id"
        );
        *self.build_id.lock().await = None;
        self.fetch(page, query).await?.ok_or_else(|| {
            StoreError::UnexpectedResponse(format!("data route for {page} not found"))
        })
    }

    /// One attempt with the current build id. `Ok(None)` means the id is
    /// stale.
    async fn fetch<T: DeserializeOwned>(
        &self,
        page: &str,
        query: &[(&str, &str)],
    ) -> Result<Option<DataPage<T>>, StoreError> {
        let id = self.build_id().await?;
        let response = self
            .http
            .get(format!("{}/_next/data/{id}/en/{page}.json", self.base_url))
            .query(query)
            .header("accept", "*/*")
            .header("x-nextjs-data", "1")
            .send()
            .await
            .map_err(transport_error)?;
        if response.status().as_u16() == 404 {
            let body = response.bytes().await.map_err(transport_error)?;
            let missing =
                serde_json::from_slice::<NotFoundBody>(&body).is_ok_and(|body| body.not_found);
            return Ok(missing.then_some(DataPage::NotFound));
        }
        Ok(Some(DataPage::Found(read_json(response).await?)))
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
}
