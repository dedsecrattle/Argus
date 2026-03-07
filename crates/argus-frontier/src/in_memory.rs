use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

use argus_common::CrawlJob;

#[derive(Clone, Default)]
pub struct InMemoryFrontier {
    queue: Arc<Mutex<VecDeque<CrawlJob>>>,
}

impl InMemoryFrontier {
    pub async fn push(&self, job: CrawlJob) {
        let mut q = self.queue.lock().await;
        q.push_back(job);
    }

    pub async fn pop(&self) -> Option<CrawlJob> {
        let mut q = self.queue.lock().await;
        q.pop_front()
    }
}
