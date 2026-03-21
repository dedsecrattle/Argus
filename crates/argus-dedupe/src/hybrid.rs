use anyhow::Result;
use async_trait::async_trait;

use crate::bloom::BloomDeduplicator;
use crate::set_trait::SeenSet;

#[cfg(feature = "redis")]
use redis::AsyncCommands;

#[cfg(not(feature = "redis"))]
use std::sync::Arc;

/// Hybrid deduplication using Bloom filter + Redis
/// - Bloom filter: Fast probabilistic "not seen" check (99% accuracy)
/// - Redis: Authoritative check for potential duplicates
pub struct HybridSeenSet {
    bloom: BloomDeduplicator,
    #[cfg(feature = "redis")]
    redis: redis::aio::ConnectionManager,
    #[cfg(feature = "redis")]
    key_prefix: String,
    #[cfg(not(feature = "redis"))]
    fallback: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
}

impl HybridSeenSet {
    #[cfg(feature = "redis")]
    pub async fn new(
        redis_url: &str,
        key_prefix: Option<String>,
        expected_urls: usize,
        false_positive_rate: f64,
    ) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = redis::aio::ConnectionManager::new(client).await?;

        Ok(Self {
            bloom: BloomDeduplicator::new(expected_urls, false_positive_rate),
            redis,
            key_prefix: key_prefix.unwrap_or_else(|| "argus:seen:".to_string()),
        })
    }

    #[cfg(not(feature = "redis"))]
    pub async fn new(
        _redis_url: &str,
        _key_prefix: Option<String>,
        expected_urls: usize,
        false_positive_rate: f64,
    ) -> Result<Self> {
        Ok(Self {
            bloom: BloomDeduplicator::new(expected_urls, false_positive_rate),
            fallback: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
        })
    }

    /// Get memory usage statistics
    pub fn stats(&self) -> HybridStats {
        HybridStats {
            bloom_memory_bytes: self.bloom.memory_usage(),
            bloom_bit_count: self.bloom.bit_count(),
            bloom_hash_count: self.bloom.hash_count(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HybridStats {
    pub bloom_memory_bytes: usize,
    pub bloom_bit_count: u64,
    pub bloom_hash_count: u32,
}

#[async_trait]
impl SeenSet for HybridSeenSet {
    #[cfg(feature = "redis")]
    async fn insert_if_new(&self, url: String) -> bool {
        // Fast path: Check bloom filter first
        if !self.bloom.might_contain(&url) {
            // Definitely new - add to both
            self.bloom.insert(&url);

            let key = format!("{}{}", self.key_prefix, url);
            let mut conn = self.redis.clone();

            // Use SETNX for atomic insert-if-new
            match conn.set_nx::<_, _, bool>(&key, 1).await {
                Ok(was_new) => was_new,
                Err(e) => {
                    eprintln!("Redis error, assuming new: {}", e);
                    true
                }
            }
        } else {
            // Might be duplicate - check Redis
            let key = format!("{}{}", self.key_prefix, url);
            let mut conn = self.redis.clone();

            match conn.set_nx::<_, _, bool>(&key, 1).await {
                Ok(was_new) => {
                    if was_new {
                        // False positive in bloom filter
                        self.bloom.insert(&url);
                    }
                    was_new
                }
                Err(e) => {
                    eprintln!("Redis error, assuming duplicate: {}", e);
                    false
                }
            }
        }
    }

    #[cfg(not(feature = "redis"))]
    async fn insert_if_new(&self, url: String) -> bool {
        // Fast path: Check bloom filter first
        if !self.bloom.might_contain(&url) {
            // Definitely new
            self.bloom.insert(&url);
            let mut set = self.fallback.write().await;
            set.insert(url);
            true
        } else {
            // Might be duplicate - check fallback set
            let mut set = self.fallback.write().await;
            if set.insert(url.clone()) {
                self.bloom.insert(&url);
                true
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn hybrid_deduplication() {
        let seen = HybridSeenSet::new("redis://localhost:6379", None, 1000, 0.01)
            .await
            .unwrap();

        assert!(
            seen.insert_if_new("https://example.com/1".to_string())
                .await
        );
        assert!(
            !seen
                .insert_if_new("https://example.com/1".to_string())
                .await
        );
        assert!(
            seen.insert_if_new("https://example.com/2".to_string())
                .await
        );
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn hybrid_stats() {
        let seen = HybridSeenSet::new("redis://localhost:6379", None, 1_000_000, 0.01)
            .await
            .unwrap();

        let stats = seen.stats();
        assert!(stats.bloom_memory_bytes > 0);
        assert!(stats.bloom_memory_bytes < 2_000_000); // Should be ~1.2MB
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn hybrid_large_scale() {
        // Test with 100M URL configuration
        let seen = HybridSeenSet::new("redis://localhost:6379", None, 100_000_000, 0.01)
            .await
            .unwrap();

        let stats = seen.stats();
        let memory_mb = stats.bloom_memory_bytes as f64 / 1_048_576.0;

        // Should use ~120MB for 100M URLs
        assert!(memory_mb < 200.0, "Memory usage: {:.2} MB", memory_mb);
    }
}
