//! A small in-memory map whose entries expire.
//!
//! Used by [`super::cached::CachedStore`] to remember store answers for a
//! few minutes. Bounded: past [`MAX_ENTRIES`] the oldest entry is dropped,
//! so a long-running process cannot grow it without limit.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

/// Most entries kept; past this the oldest is dropped.
pub const MAX_ENTRIES: usize = 256;

/// Values of type `V`, each forgotten `ttl` after it was stored.
pub struct TtlCache<V> {
    ttl: Duration,
    entries: Mutex<HashMap<String, (Instant, V)>>,
}

impl<V: Clone> TtlCache<V> {
    /// An empty cache whose entries live for `ttl`.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// The value stored under `key`, if it has not expired.
    pub async fn get(&self, key: &str) -> Option<V> {
        let entries = self.entries.lock().await;
        let (stored_at, value) = entries.get(key)?;
        (stored_at.elapsed() < self.ttl).then(|| value.clone())
    }

    /// Stores `value` under `key`, first dropping expired entries and, if
    /// still full, the oldest one.
    pub async fn insert(&self, key: String, value: V) {
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
        entries.insert(key, (Instant::now(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_fresh_value_is_returned() {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert("a".into(), 1).await;
        assert_eq!(cache.get("a").await, Some(1));
        assert_eq!(cache.get("b").await, None);
    }

    #[tokio::test]
    async fn an_expired_value_is_not() {
        let cache = TtlCache::new(Duration::ZERO);
        cache.insert("a".into(), 1).await;
        assert_eq!(cache.get("a").await, None);
    }

    #[tokio::test]
    async fn the_oldest_entry_goes_when_full() {
        let cache = TtlCache::new(Duration::from_secs(60));
        for n in 0..=MAX_ENTRIES {
            cache.insert(n.to_string(), n).await;
        }
        assert_eq!(cache.get("0").await, None);
        assert_eq!(cache.get(&MAX_ENTRIES.to_string()).await, Some(MAX_ENTRIES));
    }
}
