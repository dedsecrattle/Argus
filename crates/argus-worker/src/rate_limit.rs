use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::Mutex;

#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Wait if needed so that at least `delay_ms` has passed since the last fetch for this host.
    async fn wait_for_host(&self, host: &str, delay_ms: u64);
}

/// Per-process rate limit; each host is delayed based on local fetch times.
#[derive(Clone, Default)]
pub struct InMemoryRateLimiter {
    last: Arc<Mutex<HashMap<String, Instant>>>,
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn wait_for_host(&self, host: &str, delay_ms: u64) {
        let delay = Duration::from_millis(delay_ms);
        let map = self.last.lock().await;
        let last = map.get(host).copied();
        drop(map);
        if let Some(last) = last {
            let elapsed = last.elapsed();
            if elapsed < delay {
                tokio::time::sleep(delay - elapsed).await;
            }
        }
        let mut map = self.last.lock().await;
        map.insert(host.to_string(), Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_rate_limiter_zero_delay_does_not_block() {
        let limiter = InMemoryRateLimiter::default();
        limiter.wait_for_host("example.com", 0).await;
        limiter.wait_for_host("example.com", 0).await;
    }

    #[tokio::test]
    async fn in_memory_rate_limiter_second_call_waits() {
        let limiter = InMemoryRateLimiter::default();
        let start = std::time::Instant::now();
        limiter.wait_for_host("host", 50).await;
        limiter.wait_for_host("host", 50).await;
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(40),
            "second call should wait ~50ms, got {:?}",
            elapsed
        );
    }
}

#[cfg(feature = "redis")]
mod redis_limiter {
    use std::sync::Arc;

    use async_trait::async_trait;
    use redis::aio::MultiplexedConnection;
    use redis::AsyncCommands;
    use tokio::sync::Mutex;

    use super::RateLimiter;

    const KEY_PREFIX: &str = "argus:rate:";

    #[derive(Clone)]
    pub struct RedisRateLimiter {
        conn: Arc<Mutex<MultiplexedConnection>>,
    }

    impl RedisRateLimiter {
        pub async fn connect(redis_url: &str) -> anyhow::Result<Self> {
            let client = redis::Client::open(redis_url)?;
            let conn = client.get_multiplexed_tokio_connection().await?;
            Ok(Self {
                conn: Arc::new(Mutex::new(conn)),
            })
        }
    }

    #[async_trait]
    impl RateLimiter for RedisRateLimiter {
        async fn wait_for_host(&self, host: &str, delay_ms: u64) {
            let key = format!("{}{}", KEY_PREFIX, host);
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let mut conn = self.conn.lock().await;
            let last_ms: Option<u64> = conn.get(&key).await.ok().flatten();
            drop(conn);

            if let Some(last) = last_ms {
                let elapsed = now_ms.saturating_sub(last);
                if elapsed < delay_ms {
                    let wait_ms = delay_ms - elapsed;
                    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                }
            }

            let mut conn = self.conn.lock().await;
            let set_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let _: Result<(), _> = conn.set(&key, set_now).await;
        }
    }
}

#[cfg(feature = "redis")]
pub use redis_limiter::RedisRateLimiter;
