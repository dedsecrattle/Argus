use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;

use crate::set_trait::SeenSet;

#[derive(Clone, Default)]
pub struct SeenUrlSet {
    inner: Arc<Mutex<HashSet<String>>>,
}

#[async_trait]
impl SeenSet for SeenUrlSet {
    async fn insert_if_new(&self, url: String) -> bool {
        let mut guard = self.inner.lock().await;
        guard.insert(url)
    }
}
