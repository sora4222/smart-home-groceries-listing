//! Choosing and assembling the store clients for this process.
//!
//! The one place that names a concrete store client. Everything else holds
//! [`StoreClients`] and sees only the [`StoreClient`] trait, so swapping the
//! real websites for the fake catalogue is a setting, not a code change.

use std::sync::Arc;

use crate::config::{StoreMode, StoreSettings};

use super::cached::CachedStore;
use super::client::{BoxFuture, StoreClient};
use super::coles::ColesClient;
use super::error::StoreError;
use super::fake::FakeStore;
use super::http::build_client;
use super::product::Product;
use super::store::Store;
use super::woolworths::WoolworthsClient;

/// Every store the backend searches, in display order.
pub type StoreClients = Arc<Vec<Arc<dyn StoreClient>>>;

/// Builds the clients `settings` asks for, each behind a short cache.
pub fn build(settings: &StoreSettings) -> StoreClients {
    let clients = Store::ALL
        .into_iter()
        .map(|store| {
            let client = match settings.mode {
                StoreMode::Fake => Arc::new(FakeStore::new(store)) as Arc<dyn StoreClient>,
                StoreMode::Live => live(store, settings),
            };
            Arc::new(CachedStore::new(client, settings.cache_ttl)) as Arc<dyn StoreClient>
        })
        .collect();
    tracing::info!(mode = ?settings.mode, "store clients ready");
    Arc::new(clients)
}

/// The real client for `store`. Each store gets its own HTTP client, so the
/// stores' cookies never share a jar.
fn live(store: Store, settings: &StoreSettings) -> Arc<dyn StoreClient> {
    let http = match build_client(settings.timeout) {
        Ok(http) => http,
        Err(err) => {
            tracing::error!(%store, %err, "store HTTP client could not be built");
            return Arc::new(Broken { store, err });
        }
    };
    match store {
        Store::Woolworths => Arc::new(WoolworthsClient::new(http, settings.base_url(store))),
        Store::Coles => Arc::new(ColesClient::new(http, settings.base_url(store))),
    }
}

/// Stands in for a store whose client could not be built, so the process
/// still starts and the other store still searches.
struct Broken {
    store: Store,
    err: StoreError,
}

impl StoreClient for Broken {
    fn store(&self) -> Store {
        self.store
    }

    fn search<'a>(&'a self, _query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(async move { Err(self.err.clone()) })
    }
}
