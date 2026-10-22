//! A short-lived cache in front of a store.
//!
//! Opening the same item's price comparison twice should not search the
//! store twice: repeated identical requests are what bot protection flags.
//! [`CachedStore`] wraps any [`StoreClient`] and answers a repeated search
//! or product lookup from memory for a few minutes. Only successful answers
//! are kept, so a failure is never remembered.

use std::sync::Arc;
use std::time::Duration;

use super::client::{BoxFuture, StoreClient};
use super::error::StoreError;
use super::product::Product;
use super::store::Store;
use super::ttl_cache::TtlCache;

/// A [`StoreClient`] that remembers recent answers.
pub struct CachedStore {
    inner: Arc<dyn StoreClient>,
    searches: TtlCache<Vec<Product>>,
    products: TtlCache<Option<Product>>,
}

impl CachedStore {
    /// Wraps `inner`, remembering each answer for `ttl`.
    pub fn new(inner: Arc<dyn StoreClient>, ttl: Duration) -> Self {
        Self {
            inner,
            searches: TtlCache::new(ttl),
            products: TtlCache::new(ttl),
        }
    }
}

/// Queries that differ only in case or spacing are the same query.
fn cache_key(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

impl StoreClient for CachedStore {
    fn store(&self) -> Store {
        self.inner.store()
    }

    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(async move {
            let key = cache_key(query);
            if let Some(products) = self.searches.get(&key).await {
                tracing::debug!(store = %self.store(), query = %key, "store search answered from cache");
                return Ok(products);
            }
            let products = self.inner.search(query).await?;
            self.searches.insert(key, products.clone()).await;
            Ok(products)
        })
    }

    fn product<'a>(
        &'a self,
        product_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Product>, StoreError>> {
        Box::pin(async move {
            let key = product_id.trim().to_string();
            if let Some(found) = self.products.get(&key).await {
                tracing::debug!(store = %self.store(), product_id = %key, "product lookup answered from cache");
                return Ok(found);
            }
            let found = self.inner.product(product_id).await?;
            self.products.insert(key, found.clone()).await;
            Ok(found)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Counts searches and fails when told to.
    struct Counting {
        calls: AtomicUsize,
        fail: bool,
    }

    impl StoreClient for Counting {
        fn store(&self) -> Store {
            Store::Coles
        }
        fn search<'a>(&'a self, _: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let fail = self.fail;
            Box::pin(async move {
                if fail {
                    Err(StoreError::Unreachable("down".into()))
                } else {
                    Ok(Vec::new())
                }
            })
        }
        fn product<'a>(&'a self, _: &'a str) -> BoxFuture<'a, Result<Option<Product>, StoreError>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let fail = self.fail;
            Box::pin(async move {
                if fail {
                    Err(StoreError::Unreachable("down".into()))
                } else {
                    Ok(None)
                }
            })
        }
    }

    fn counting(fail: bool) -> Arc<Counting> {
        Arc::new(Counting {
            calls: AtomicUsize::new(0),
            fail,
        })
    }

    #[tokio::test]
    async fn a_repeated_query_is_answered_from_memory() {
        let inner = counting(false);
        let cached = CachedStore::new(inner.clone(), Duration::from_secs(60));
        cached.search("Milk").await.unwrap();
        cached.search("  milk ").await.unwrap();
        assert_eq!(inner.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn an_expired_entry_searches_again() {
        let inner = counting(false);
        let cached = CachedStore::new(inner.clone(), Duration::ZERO);
        cached.search("milk").await.unwrap();
        cached.search("milk").await.unwrap();
        assert_eq!(inner.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_failure_is_not_remembered() {
        let inner = counting(true);
        let cached = CachedStore::new(inner.clone(), Duration::from_secs(60));
        assert!(cached.search("milk").await.is_err());
        assert!(cached.search("milk").await.is_err());
        assert_eq!(inner.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_repeated_lookup_is_answered_from_memory() {
        let inner = counting(false);
        let cached = CachedStore::new(inner.clone(), Duration::from_secs(60));
        assert_eq!(cached.product("123").await.unwrap(), None);
        assert_eq!(cached.product("123").await.unwrap(), None);
        assert_eq!(inner.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_failed_lookup_is_not_remembered() {
        let inner = counting(true);
        let cached = CachedStore::new(inner.clone(), Duration::from_secs(60));
        assert!(cached.product("123").await.is_err());
        assert!(cached.product("123").await.is_err());
        assert_eq!(inner.calls.load(Ordering::SeqCst), 2);
    }
}
