use std::time::Duration;

use anyhow::Result;
use argus_common::{CrawlJob, FetchResult};
use reqwest::Client;

use crate::error::FetchError;
use crate::retry::RetryConfig;

#[derive(Clone, Debug)]
pub struct FetcherConfig {
    pub user_agent: String,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_redirects: usize,
    pub retry_config: RetryConfig,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            user_agent: "argus/0.1".to_string(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_redirects: 10,
            retry_config: RetryConfig::default(),
        }
    }
}

#[derive(Clone)]
pub struct HttpFetcher {
    client: Client,
    config: FetcherConfig,
}

impl HttpFetcher {
    pub fn new() -> Result<Self> {
        Self::with_config(FetcherConfig::default())
    }

    pub fn with_config(config: FetcherConfig) -> Result<Self> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()?;

        Ok(Self { client, config })
    }

    pub async fn fetch(&self, job: &CrawlJob) -> Result<FetchResult> {
        let mut attempts = 0;
        let max_retries = self.config.retry_config.max_retries;

        loop {
            match self.fetch_once(job).await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if !err.is_retryable() || attempts >= max_retries {
                        return Err(anyhow::anyhow!(err));
                    }

                    let backoff = self.config.retry_config.calculate_backoff(attempts);
                    tracing::debug!(
                        "retry attempt {} after {:?} for {}: {}",
                        attempts + 1,
                        backoff,
                        job.url,
                        err
                    );

                    tokio::time::sleep(backoff).await;
                    attempts += 1;
                }
            }
        }
    }

    async fn fetch_once(&self, job: &CrawlJob) -> Result<FetchResult, FetchError> {
        let response = self
            .client
            .get(&job.url)
            .send()
            .await
            .map_err(|e| FetchError::from_reqwest(&e))?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response
            .bytes()
            .await
            .map_err(|e| FetchError::from_reqwest(&e))?;

        Ok(FetchResult {
            url: job.url.clone(),
            final_url,
            status,
            content_type,
            body,
        })
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new().expect("failed to create default HttpFetcher")
    }
}
