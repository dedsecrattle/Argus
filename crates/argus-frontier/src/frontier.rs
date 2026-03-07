use async_trait::async_trait;

use argus_common::CrawlJob;

#[async_trait]
pub trait Frontier: Send + Sync {
    async fn push(&self, job: CrawlJob);
    async fn pop(&self) -> Option<CrawlJob>;
}
