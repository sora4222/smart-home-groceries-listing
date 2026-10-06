//! Coles product search and lookup through the website's Next.js data
//! routes ([`data_route`]).
//!
//! - Search: `search/products?q=` — the search page's data.
//! - Lookup: `product/<slug>-<id>` — a product page's data. Coles answers
//!   any slug ending in the id with a redirect to the real slug
//!   (`__N_REDIRECT`), so the lookup asks for `product/x-<id>` and follows
//!   that redirect once. Captured live on 2026-10-06.

mod build_id;
mod data_route;
mod mapping;
mod wire;

use super::client::{BoxFuture, StoreClient};
use super::error::StoreError;
use super::product::Product;
use super::product_id;
use super::store::Store;
use data_route::{DataPage, DataRoute};
use wire::{ProductPage, SearchPage};

/// Talks to Coles.
pub struct ColesClient {
    route: DataRoute,
}

impl ColesClient {
    /// A client for `base_url` (the real site, or a mock server in tests).
    pub fn new(http: wreq::Client, base_url: &str) -> Self {
        Self {
            route: DataRoute::new(http, base_url),
        }
    }

    /// The search page's products for `query`.
    async fn search_products(&self, query: &str) -> Result<Vec<Product>, StoreError> {
        let page: DataPage<SearchPage> = self.route.get("search/products", &[("q", query)]).await?;
        Ok(match page {
            DataPage::Found(page) => mapping::products(page, self.route.base_url()),
            DataPage::NotFound => Vec::new(),
        })
    }

    /// One product page, following Coles' redirect to the real slug once.
    async fn product_page(&self, id: &str) -> Result<Option<Product>, StoreError> {
        let mut page_path = format!("product/x-{id}");
        for _ in 0..2 {
            let page: DataPage<ProductPage> = self.route.get(&page_path, &[]).await?;
            let DataPage::Found(page) = page else {
                return Ok(None);
            };
            if let Some(product) = page.page_props.product {
                return Ok(Some(mapping::product(product, self.route.base_url())));
            }
            let target = page
                .page_props
                .redirect
                .as_deref()
                .and_then(|to| redirect_path(to, id));
            match target {
                Some(next) => page_path = next,
                None => break,
            }
        }
        Err(StoreError::UnexpectedResponse(
            "product page had neither a product nor a usable redirect".into(),
        ))
    }
}

/// The data route path for a redirect to `/product/<slug>-<id>`, or `None`
/// if the redirect goes anywhere else. The slug is checked because it is
/// put into a URL path.
fn redirect_path(to: &str, id: &str) -> Option<String> {
    let slug = to.strip_prefix("/product/")?.trim_end_matches('/');
    let safe = !slug.is_empty()
        && slug.len() <= 200
        && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    (safe && slug.ends_with(&format!("-{id}"))).then(|| format!("product/{slug}"))
}

impl StoreClient for ColesClient {
    fn store(&self) -> Store {
        Store::Coles
    }

    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(self.search_products(query))
    }

    fn product<'a>(
        &'a self,
        product_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Product>, StoreError>> {
        Box::pin(async move {
            if !product_id::is_valid(product_id) {
                return Ok(None);
            }
            self.product_page(product_id).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::redirect_path;

    #[test]
    fn follows_a_redirect_to_the_same_product() {
        assert_eq!(
            redirect_path("/product/coles-full-cream-milk-3l-8150288", "8150288").as_deref(),
            Some("product/coles-full-cream-milk-3l-8150288")
        );
    }

    #[test]
    fn refuses_a_redirect_anywhere_else() {
        assert_eq!(redirect_path("/product/other-thing-123", "8150288"), None);
        assert_eq!(redirect_path("/browse/dairy-8150288", "8150288"), None);
        assert_eq!(redirect_path("/product/../../x-8150288", "8150288"), None);
    }
}
