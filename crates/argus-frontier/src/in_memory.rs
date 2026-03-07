use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;

use argus_common::CrawlJob;

use crate::frontier::Frontier;

#[derive(Clone, Default)]
pub struct InMemoryFrontier {
    queue: Arc<Mutex<VecDeque<CrawlJob>>>,
}

#[async_trait]
impl Frontier for InMemoryFrontier {
    async fn push(&self, job: CrawlJob) {
        let mut q = self.queue.lock().await;
        q.push_back(job);
    }

    async fn pop(&self) -> Option<CrawlJob> {
        let mut q = self.queue.lock().await;
        q.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(url: &str, depth: u16) -> CrawlJob {
        CrawlJob {
            url: url.to_string(),
            normalized_url: url.to_string(),
            host: "example.com".to_string(),
            depth,
        }
    }

    #[tokio::test]
    async fn push_pop_fifo_order() {
        let frontier = InMemoryFrontier::default();
        frontier.push(job("https://a.com", 0)).await;
        frontier.push(job("https://b.com", 1)).await;
        let first = frontier.pop().await.unwrap();
        let second = frontier.pop().await.unwrap();
        assert_eq!(first.url, "https://a.com");
        assert_eq!(second.url, "https://b.com");
    }

    #[tokio::test]
    async fn pop_empty_returns_none() {
        let frontier = InMemoryFrontier::default();
        assert!(frontier.pop().await.is_none());
    }
}
