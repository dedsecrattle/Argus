use std::time::Duration;

use anyhow::Result;
use argus_common::{CrawlJob, FetchResult};
use reqwest::Client;

#[derive(Clone)]
pub struct HttpFetcher {
    client: Client,
}

impl HttpFetcher {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("argus/0.1")
            .redirect(reqwest::redirect::Policy::limited(10))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .build()?;

        Ok(Self { client })
    }

    pub async fn fetch(&self, job: &CrawlJob) -> Result<FetchResult> {
        let response = self.client.get(&job.url).send().await?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response.bytes().await?;

        Ok(FetchResult {
            url: job.url.clone(),
            final_url,
            status,
            content_type,
            body,
        })
    }
}
