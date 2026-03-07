use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tokio::sync::Mutex;

use crate::set_trait::SeenSet;

const DEFAULT_SEEN_KEY: &str = "argus:seen";

#[derive(Clone)]
pub struct RedisSeenSet {
    conn: Arc<Mutex<MultiplexedConnection>>,
    set_key: String,
}

impl RedisSeenSet {
    pub fn new(conn: MultiplexedConnection, set_key: Option<String>) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            set_key: set_key.unwrap_or_else(|| DEFAULT_SEEN_KEY.to_string()),
        }
    }

    pub async fn connect(redis_url: &str, set_key: Option<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self::new(conn, set_key))
    }
}

#[async_trait]
impl SeenSet for RedisSeenSet {
    async fn insert_if_new(&self, url: String) -> bool {
        let mut conn = self.conn.lock().await;
        let key = self.set_key.clone();
        let added: i32 = conn.sadd(&key, &url).await.unwrap_or(0);
        added == 1
    }
}
