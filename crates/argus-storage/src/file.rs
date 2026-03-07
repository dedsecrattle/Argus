use std::path::Path;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Serialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use argus_common::{CrawlJob, FetchResult};

use crate::storage_trait::{Storage, url_to_fragment};

#[derive(Serialize)]
struct PageMeta {
    url: String,
    final_url: String,
    status: u16,
    content_type: Option<String>,
    depth: u16,
    body_path: String,
    fetched_at_ms: u64,
}

/// Writes each fetch to a directory: `base_dir/page/<fragment>.json` (metadata) and
/// `base_dir/body/<fragment>.bin` (raw body).
#[derive(Clone)]
pub struct FileStorage {
    base_path: std::path::PathBuf,
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub async fn ensure_dirs(&self) -> Result<()> {
        let page_dir = self.base_path.join("page");
        let body_dir = self.base_path.join("body");
        fs::create_dir_all(&page_dir)
            .await
            .context("create page dir")?;
        fs::create_dir_all(&body_dir)
            .await
            .context("create body dir")?;
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn record_fetch(&self, job: &CrawlJob, result: &FetchResult) -> Result<()> {
        self.ensure_dirs().await?;

        let fragment = url_to_fragment(&job.normalized_url);
        let body_path = format!("body/{}.bin", fragment);
        let body_full = self.base_path.join(&body_path);
        let meta_path = self.base_path.join("page").join(format!("{}.json", fragment));

        fs::write(&body_full, &result.body)
            .await
            .context("write body file")?;

        let meta = PageMeta {
            url: job.url.clone(),
            final_url: result.final_url.clone(),
            status: result.status,
            content_type: result.content_type.clone(),
            depth: job.depth,
            body_path,
            fetched_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        let json = serde_json::to_string_pretty(&meta).context("serialize meta")?;
        let mut f = fs::File::create(&meta_path).await.context("create meta file")?;
        f.write_all(json.as_bytes()).await.context("write meta file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use argus_common::{CrawlJob, FetchResult};

    use super::*;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("argus-storage-test-{}", std::process::id()))
    }

    #[tokio::test]
    async fn record_fetch_creates_page_and_body_files() {
        let base = temp_dir();
        let _ = std::fs::remove_dir_all(&base);
        let storage = FileStorage::new(&base);
        storage.ensure_dirs().await.unwrap();

        let job = CrawlJob {
            url: "https://example.com/".to_string(),
            normalized_url: "https://example.com/".to_string(),
            host: "example.com".to_string(),
            depth: 0,
        };
        let result = FetchResult {
            url: job.url.clone(),
            final_url: "https://example.com/".to_string(),
            status: 200,
            content_type: Some("text/html".to_string()),
            body: bytes::Bytes::from_static(b"<html>body</html>"),
        };

        storage.record_fetch(&job, &result).await.unwrap();

        let fragment = crate::storage_trait::url_to_fragment(&job.normalized_url);
        let meta_path = base.join("page").join(format!("{}.json", fragment));
        let body_path = base.join("body").join(format!("{}.bin", fragment));

        assert!(meta_path.exists(), "metadata file should exist");
        assert!(body_path.exists(), "body file should exist");

        let meta_json = std::fs::read_to_string(&meta_path).unwrap();
        assert!(meta_json.contains("https://example.com/"));
        assert!(meta_json.contains("\"status\": 200"));

        let body = std::fs::read(&body_path).unwrap();
        assert_eq!(body, b"<html>body</html>");

        let _ = std::fs::remove_dir_all(&base);
    }
}
