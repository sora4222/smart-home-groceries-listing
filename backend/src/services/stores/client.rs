//! The contract every store integration satisfies.

use std::future::Future;
use std::pin::Pin;

use super::error::StoreError;
use super::product::Product;
use super::store::Store;

/// A boxed future, so [`StoreClient`] stays usable behind `dyn`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// One retailer's product search.
///
/// The search service and, later, the order optimiser only ever see this
/// trait — never a store's URLs, cookies or JSON. Delivery windows and
/// checkout will be added here when those features are built.
pub trait StoreClient: Send + Sync {
    /// Which store this client talks to.
    fn store(&self) -> Store;

    /// Searches the store for `query`, returning its products in the store's
    /// own order.
    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>>;
}
