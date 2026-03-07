use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct SeenUrlSet {
    inner: Arc<Mutex<HashSet<String>>>,
}

impl SeenUrlSet {
    pub async fn insert_if_new(&self, url: String) -> bool {
        let mut guard = self.inner.lock().await;
        guard.insert(url)
    }
}
