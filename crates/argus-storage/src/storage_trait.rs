use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::Result;
use async_trait::async_trait;

use argus_common::{CrawlJob, FetchResult};

#[async_trait]
pub trait Storage: Send + Sync {
    /// Persist a fetch result. Called after each successful fetch.
    async fn record_fetch(&self, job: &CrawlJob, result: &FetchResult) -> Result<()>;
}

/// No-op storage; does not persist anything.
#[derive(Clone, Default)]
pub struct NoopStorage;

#[async_trait]
impl Storage for NoopStorage {
    async fn record_fetch(&self, _job: &CrawlJob, _result: &FetchResult) -> Result<()> {
        Ok(())
    }
}

/// Safe filename fragment from a URL (hash-based).
pub fn url_to_fragment(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_to_fragment_deterministic() {
        let a = url_to_fragment("https://example.com/page");
        let b = url_to_fragment("https://example.com/page");
        assert_eq!(a, b);
    }

    #[test]
    fn url_to_fragment_different_urls_differ() {
        let a = url_to_fragment("https://example.com/a");
        let b = url_to_fragment("https://example.com/b");
        assert_ne!(a, b);
    }

    #[test]
    fn url_to_fragment_hex_and_fixed_width() {
        let s = url_to_fragment("https://x.org");
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
