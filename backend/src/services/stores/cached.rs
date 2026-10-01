//! A short-lived cache in front of a store.
//!
//! Opening the same item's price comparison twice should not search the
//! store twice: repeated identical requests are what bot protection flags.
//! [`CachedStore`] wraps any [`StoreClient`] and answers a repeated query
//! from memory for a few minutes. Only successful searches are kept, so a
//! failure is never remembered.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::client::{BoxFuture, StoreClient};
use super::error::StoreError;
use super::product::Product;
use super::store::Store;

/// Most queries remembered per store; past this the oldest are dropped.
const MAX_ENTRIES: usize = 256;

/// A [`StoreClient`] that remembers recent results.
pub struct CachedStore {
    inner: Arc<dyn StoreClient>,
    ttl: Duration,
    entries: Mutex<HashMap<String, (Instant, Vec<Product>)>>,
}

impl CachedStore {
    /// Wraps `inner`, remembering each result for `ttl`.
    pub fn new(inner: Arc<dyn StoreClient>, ttl: Duration) -> Self {
        Self {
            inner,
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// A fresh remembered result for `key`, if there is one.
    async fn remembered(&self, key: &str) -> Option<Vec<Product>> {
        let entries = self.entries.lock().await;
        let (stored_at, products) = entries.get(key)?;
        (stored_at.elapsed() < self.ttl).then(|| products.clone())
    }

    /// Stores a result, evicting expired entries and, if still full, the
    /// oldest one.
    async fn remember(&self, key: String, products: Vec<Product>) {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, (stored_at, _)| stored_at.elapsed() < self.ttl);
        if entries.len() >= MAX_ENTRIES {
            let oldest = entries
                .iter()
                .min_by_key(|(_, (stored_at, _))| *stored_at)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                entries.remove(&oldest);
            }
        }
        entries.insert(key, (Instant::now(), products));
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
            if let Some(products) = self.remembered(&key).await {
                tracing::debug!(store = %self.store(), query = %key, "store search answered from cache");
                return Ok(products);
            }
            let products = self.inner.search(query).await?;
            self.remember(key, products.clone()).await;
            Ok(products)
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
}
