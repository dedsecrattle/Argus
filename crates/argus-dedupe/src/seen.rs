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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn insert_if_new_returns_true_for_new_url() {
        let set = SeenUrlSet::default();
        let inserted = set.insert_if_new("https://example.com/".to_string()).await;
        assert!(inserted);
    }

    #[tokio::test]
    async fn insert_if_new_returns_false_for_seen_url() {
        let set = SeenUrlSet::default();
        let url = "https://example.com/page".to_string();
        set.insert_if_new(url.clone()).await;
        let inserted = set.insert_if_new(url).await;
        assert!(!inserted);
    }
}
