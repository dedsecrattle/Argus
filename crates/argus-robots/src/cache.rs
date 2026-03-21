use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use url::Url;

use crate::parser::RobotsTxt;

#[derive(Clone)]
struct CachedRobots {
    robots: RobotsTxt,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct RobotsCache {
    cache: Arc<RwLock<HashMap<String, CachedRobots>>>,
    client: reqwest::Client,
    user_agent: String,
    ttl: Duration,
}

impl RobotsCache {
    pub fn new(user_agent: String, ttl: Duration) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            client,
            user_agent,
            ttl,
        })
    }

    pub async fn is_allowed(&self, url: &str) -> Result<bool> {
        let parsed = Url::parse(url).context("invalid URL")?;
        let origin = format!(
            "{}://{}{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or(""),
            if let Some(port) = parsed.port() {
                format!(":{}", port)
            } else {
                String::new()
            }
        );

        let robots = self.get_robots(&origin).await?;
        let path = parsed.path();
        Ok(robots.is_allowed(path))
    }

    pub async fn get_crawl_delay(&self, url: &str) -> Result<Option<Duration>> {
        let parsed = Url::parse(url).context("invalid URL")?;
        let origin = format!(
            "{}://{}{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or(""),
            if let Some(port) = parsed.port() {
                format!(":{}", port)
            } else {
                String::new()
            }
        );

        let robots = self.get_robots(&origin).await?;
        Ok(robots.crawl_delay())
    }

    async fn get_robots(&self, origin: &str) -> Result<RobotsTxt> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(origin) {
                if cached.fetched_at.elapsed() < self.ttl {
                    return Ok(cached.robots.clone());
                }
            }
        }

        let robots_url = format!("{}/robots.txt", origin);
        tracing::debug!("fetching robots.txt from {}", robots_url);

        let robots = match self.fetch_robots(&robots_url).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("failed to fetch robots.txt from {}: {}", robots_url, e);
                RobotsTxt::parse("", &self.user_agent)
            }
        };

        let mut cache = self.cache.write().await;
        cache.insert(
            origin.to_string(),
            CachedRobots {
                robots: robots.clone(),
                fetched_at: Instant::now(),
            },
        );

        Ok(robots)
    }

    async fn fetch_robots(&self, url: &str) -> Result<RobotsTxt> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .context("failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("non-success status: {}", response.status());
        }

        let content = response.text().await.context("failed to read response")?;
        Ok(RobotsTxt::parse(&content, &self.user_agent))
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn cache_robots_txt() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/robots.txt"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("User-agent: *\nDisallow: /admin/\n"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let cache = RobotsCache::new("TestBot".to_string(), Duration::from_secs(3600)).unwrap();

        let url1 = format!("{}/page", mock_server.uri());
        let url2 = format!("{}/admin/secret", mock_server.uri());

        assert!(cache.is_allowed(&url1).await.unwrap());
        assert!(!cache.is_allowed(&url2).await.unwrap());

        assert!(cache.is_allowed(&url1).await.unwrap());
    }

    #[tokio::test]
    async fn handle_missing_robots_txt() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/robots.txt"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let cache = RobotsCache::new("TestBot".to_string(), Duration::from_secs(3600)).unwrap();

        let url = format!("{}/any-page", mock_server.uri());
        assert!(cache.is_allowed(&url).await.unwrap());
    }

    #[tokio::test]
    async fn respect_crawl_delay() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/robots.txt"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("User-agent: *\nCrawl-delay: 1.5\n"),
            )
            .mount(&mock_server)
            .await;

        let cache = RobotsCache::new("TestBot".to_string(), Duration::from_secs(3600)).unwrap();

        let url = format!("{}/page", mock_server.uri());
        let delay = cache.get_crawl_delay(&url).await.unwrap();
        assert_eq!(delay, Some(Duration::from_secs_f64(1.5)));
    }

    #[tokio::test]
    async fn cache_expiration() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/robots.txt"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("User-agent: *\nDisallow: /\n"),
            )
            .expect(2)
            .mount(&mock_server)
            .await;

        let cache = RobotsCache::new("TestBot".to_string(), Duration::from_millis(100)).unwrap();

        let url = format!("{}/page", mock_server.uri());

        assert!(!cache.is_allowed(&url).await.unwrap());

        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(!cache.is_allowed(&url).await.unwrap());
    }
}
