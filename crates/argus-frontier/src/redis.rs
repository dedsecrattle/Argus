use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tokio::sync::Mutex;

use argus_common::CrawlJob;

use crate::frontier::Frontier;

const DEFAULT_QUEUE_KEY: &str = "argus:frontier";

#[derive(Clone)]
pub struct RedisFrontier {
    conn: Arc<Mutex<MultiplexedConnection>>,
    queue_key: String,
}

impl RedisFrontier {
    pub fn new(conn: MultiplexedConnection, queue_key: Option<String>) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            queue_key: queue_key.unwrap_or_else(|| DEFAULT_QUEUE_KEY.to_string()),
        }
    }

    pub async fn connect(redis_url: &str, queue_key: Option<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self::new(conn, queue_key))
    }
}

#[async_trait]
impl Frontier for RedisFrontier {
    async fn push(&self, job: CrawlJob) {
        let serialized = match serde_json::to_string(&job) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut conn = self.conn.lock().await;
        let key = self.queue_key.clone();
        let _: Result<(), redis::RedisError> =
            redis::cmd("LPUSH").arg(&key).arg(serialized).query_async(&mut *conn).await;
    }

    async fn pop(&self) -> Option<CrawlJob> {
        let mut conn = self.conn.lock().await;
        let key = self.queue_key.clone();
        // RPOP is non-blocking so we release the lock quickly; the worker loop retries after a short sleep when empty.
        let serialized: Option<String> = conn.rpop(&key, None).await.ok().flatten();
        serialized.and_then(|s| serde_json::from_str(&s).ok())
    }
}
