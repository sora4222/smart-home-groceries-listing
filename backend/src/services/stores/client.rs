//! The contract every store integration satisfies.

use std::future::Future;
use std::pin::Pin;

use super::error::StoreError;
use super::product::Product;
use super::store::Store;

/// A boxed future, so [`StoreClient`] stays usable behind `dyn`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// One retailer's catalogue: finding products by words, and looking one up
/// by the store's own id.
///
/// The search service, the product lookup route and the order optimiser
/// only ever see this trait — never a store's URLs, cookies or JSON — so a
/// store is added by implementing it and listing it in
/// [`super::registry`]. Putting products in a trolley happens in the
/// household's own logged-in browser tab instead (see
/// `docs/features/FEATURE_TROLLEY_HANDOFF.md`).
pub trait StoreClient: Send + Sync {
    /// Which store this client talks to.
    fn store(&self) -> Store;

    /// Searches the store for `query`, returning its products in the store's
    /// own order.
    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>>;

    /// Looks up one product by the store's own id (Woolworths stockcode,
    /// Coles product id). `Ok(None)` when the store does not sell it, or the
    /// id is not one of this store's ids ([`super::product_id::is_valid`]).
    fn product<'a>(
        &'a self,
        product_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Product>, StoreError>>;
}
