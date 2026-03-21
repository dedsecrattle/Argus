#[cfg(feature = "redis")]
use anyhow::{Context, Result};
#[cfg(feature = "redis")]
use argus_common::types::CrawlJob;
#[cfg(feature = "redis")]
use async_trait::async_trait;
#[cfg(feature = "redis")]
use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
#[cfg(feature = "redis")]
use std::collections::HashMap;

#[cfg(feature = "redis")]
type XAutoClaimResult = Vec<HashMap<String, Vec<(String, HashMap<String, String>)>>>;

#[cfg(feature = "redis")]
use crate::frontier::Frontier;

/// Redis Streams-based frontier for high-throughput job distribution
/// Provides better backpressure handling and consumer groups
#[cfg(feature = "redis")]
pub struct StreamFrontier {
    conn: ConnectionManager,
    stream_key: String,
    consumer_group: String,
    consumer_name: String,
    batch_size: usize,
}

#[cfg(feature = "redis")]
impl StreamFrontier {
    /// Create a new Redis Streams frontier
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    /// * `stream_key` - Stream name (default: "argus:jobs")
    /// * `consumer_group` - Consumer group name (default: "workers")
    /// * `consumer_name` - Unique consumer identifier
    pub async fn new(
        redis_url: &str,
        stream_key: Option<String>,
        consumer_group: Option<String>,
        consumer_name: String,
    ) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        let stream_key = stream_key.unwrap_or_else(|| "argus:jobs".to_string());
        let consumer_group = consumer_group.unwrap_or_else(|| "workers".to_string());

        let mut frontier = Self {
            conn,
            stream_key,
            consumer_group,
            consumer_name,
            batch_size: 10,
        };

        // Create consumer group if it doesn't exist
        frontier.ensure_consumer_group().await?;

        Ok(frontier)
    }

    /// Set batch size for reading from stream
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    async fn ensure_consumer_group(&mut self) -> Result<()> {
        let result: Result<String, RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.stream_key)
            .arg(&self.consumer_group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut self.conn)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // Ignore "BUSYGROUP" error (group already exists)
                if e.to_string().contains("BUSYGROUP") {
                    Ok(())
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Acknowledge a job as processed
    pub async fn ack(&mut self, message_id: &str) -> Result<()> {
        let _: i64 = redis::cmd("XACK")
            .arg(&self.stream_key)
            .arg(&self.consumer_group)
            .arg(message_id)
            .query_async(&mut self.conn)
            .await
            .context("Failed to acknowledge message")?;

        Ok(())
    }

    /// Get pending messages count for this consumer
    pub async fn pending_count(&mut self) -> Result<usize> {
        let result: Vec<redis::Value> = redis::cmd("XPENDING")
            .arg(&self.stream_key)
            .arg(&self.consumer_group)
            .query_async(&mut self.conn)
            .await?;

        if let Some(redis::Value::Int(count)) = result.first() {
            Ok(*count as usize)
        } else {
            Ok(0)
        }
    }

    /// Get stream length
    pub async fn stream_len(&mut self) -> Result<usize> {
        let len: usize = self.conn.xlen(&self.stream_key).await?;
        Ok(len)
    }

    /// Claim abandoned messages (from dead consumers)
    pub async fn claim_abandoned(
        &mut self,
        idle_time_ms: usize,
    ) -> Result<Vec<(String, CrawlJob)>> {
        let result: XAutoClaimResult = redis::cmd("XAUTOCLAIM")
            .arg(&self.stream_key)
            .arg(&self.consumer_group)
            .arg(&self.consumer_name)
            .arg(idle_time_ms)
            .arg("0-0")
            .arg("COUNT")
            .arg(self.batch_size)
            .query_async(&mut self.conn)
            .await?;

        let mut jobs = Vec::new();
        for entry in result {
            for (_, messages) in entry {
                for (msg_id, fields) in messages {
                    if let Some(job_json) = fields.get("job") {
                        if let Ok(job) = serde_json::from_str::<CrawlJob>(job_json) {
                            jobs.push((msg_id, job));
                        }
                    }
                }
            }
        }

        Ok(jobs)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl Frontier for StreamFrontier {
    async fn push(&self, job: CrawlJob) {
        let job_json = match serde_json::to_string(&job) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize job: {}", e);
                return;
            }
        };

        let mut conn = self.conn.clone();
        let _: Result<String, _> = conn
            .xadd(&self.stream_key, "*", &[("job", job_json.as_str())])
            .await;
    }

    async fn pop(&self) -> Option<CrawlJob> {
        let mut conn = self.conn.clone();

        // Read from consumer group
        let result: Result<
            Vec<HashMap<String, Vec<(String, HashMap<String, String>)>>>,
            RedisError,
        > = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(&self.consumer_group)
            .arg(&self.consumer_name)
            .arg("COUNT")
            .arg(1)
            .arg("BLOCK")
            .arg(1000) // 1 second timeout
            .arg("STREAMS")
            .arg(&self.stream_key)
            .arg(">")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(streams) => {
                for stream in streams {
                    for (_, messages) in stream {
                        for (_msg_id, fields) in messages {
                            if let Some(job_json) = fields.get("job") {
                                if let Ok(job) = serde_json::from_str::<CrawlJob>(job_json) {
                                    // Store message ID for later acknowledgment
                                    // In production, you'd want to track this properly
                                    return Some(job);
                                }
                            }
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }
}

#[cfg(not(feature = "redis"))]
pub struct StreamFrontier;

#[cfg(not(feature = "redis"))]
impl StreamFrontier {
    pub async fn new(
        _redis_url: &str,
        _stream_key: Option<String>,
        _consumer_group: Option<String>,
        _consumer_name: String,
    ) -> anyhow::Result<Self> {
        anyhow::bail!("Redis Streams not enabled. Compile with 'redis' feature.")
    }
}

#[cfg(all(test, feature = "redis"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn stream_frontier_basic() {
        let frontier = StreamFrontier::new(
            "redis://localhost:6379",
            Some("test:stream".to_string()),
            Some("test:group".to_string()),
            "consumer1".to_string(),
        )
        .await
        .unwrap();

        let job = CrawlJob {
            url: "https://example.com".to_string(),
            normalized_url: "https://example.com".to_string(),
            host: "example.com".to_string(),
            depth: 0,
        };

        frontier.push(job.clone()).await;

        let popped = frontier.pop().await;
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().url, job.url);
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn stream_frontier_consumer_groups() {
        let consumer1 = StreamFrontier::new(
            "redis://localhost:6379",
            Some("test:stream2".to_string()),
            Some("test:group2".to_string()),
            "consumer1".to_string(),
        )
        .await
        .unwrap();

        let consumer2 = StreamFrontier::new(
            "redis://localhost:6379",
            Some("test:stream2".to_string()),
            Some("test:group2".to_string()),
            "consumer2".to_string(),
        )
        .await
        .unwrap();

        // Push multiple jobs
        for i in 0..10 {
            let job = CrawlJob {
                url: format!("https://example.com/{}", i),
                normalized_url: format!("https://example.com/{}", i),
                host: "example.com".to_string(),
                depth: 0,
            };
            consumer1.push(job).await;
        }

        // Both consumers should get different jobs
        let job1 = consumer1.pop().await;
        let job2 = consumer2.pop().await;

        assert!(job1.is_some());
        assert!(job2.is_some());
        assert_ne!(job1.unwrap().url, job2.unwrap().url);
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn stream_frontier_stats() {
        let mut frontier = StreamFrontier::new(
            "redis://localhost:6379",
            Some("test:stream3".to_string()),
            Some("test:group3".to_string()),
            "consumer1".to_string(),
        )
        .await
        .unwrap();

        // Push jobs
        for i in 0..5 {
            let job = CrawlJob {
                url: format!("https://example.com/{}", i),
                normalized_url: format!("https://example.com/{}", i),
                host: "example.com".to_string(),
                depth: 0,
            };
            frontier.push(job).await;
        }

        let len = frontier.stream_len().await.unwrap();
        assert_eq!(len, 5);
    }
}
