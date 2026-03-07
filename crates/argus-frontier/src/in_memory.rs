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
