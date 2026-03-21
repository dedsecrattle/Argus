#[cfg(feature = "s3")]
use anyhow::{Context, Result};
#[cfg(feature = "s3")]
use async_trait::async_trait;
#[cfg(feature = "s3")]
use aws_sdk_s3::{primitives::ByteStream, Client};
#[cfg(feature = "s3")]
use bytes::Bytes;
#[cfg(feature = "s3")]
use std::sync::Arc;

#[cfg(feature = "s3")]
use crate::storage_trait::{url_to_fragment, Storage};
#[cfg(feature = "s3")]
use argus_common::types::{CrawlJob, FetchResult};

/// S3-compatible object storage backend
/// Works with AWS S3, MinIO, DigitalOcean Spaces, etc.
#[cfg(feature = "s3")]
pub struct S3Storage {
    client: Arc<Client>,
    bucket: String,
    prefix: String,
}

#[cfg(feature = "s3")]
impl S3Storage {
    /// Create a new S3 storage backend
    ///
    /// # Arguments
    /// * `bucket` - S3 bucket name
    /// * `prefix` - Optional prefix for all keys (e.g., "crawl/")
    pub async fn new(bucket: String, prefix: Option<String>) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Ok(Self {
            client: Arc::new(client),
            bucket,
            prefix: prefix.unwrap_or_default(),
        })
    }

    /// Create with custom endpoint (for MinIO, etc.)
    pub async fn new_with_endpoint(
        bucket: String,
        prefix: Option<String>,
        endpoint_url: String,
    ) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint_url)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client: Arc::new(client),
            bucket,
            prefix: prefix.unwrap_or_default(),
        })
    }

    fn metadata_key(&self, fragment: &str) -> String {
        format!("{}page/{}.json", self.prefix, fragment)
    }

    fn body_key(&self, fragment: &str) -> String {
        format!("{}body/{}.bin", self.prefix, fragment)
    }

    /// Get an object from S3
    pub async fn get_object(&self, key: &str) -> Result<Bytes> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("Failed to get object from S3")?;

        let data = response
            .body
            .collect()
            .await
            .context("Failed to read S3 object body")?;

        Ok(data.into_bytes())
    }

    /// Check if bucket exists and is accessible
    pub async fn verify_bucket(&self) -> Result<()> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .context("Failed to access S3 bucket")?;

        Ok(())
    }

    /// List objects with a given prefix
    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        let full_prefix = format!("{}{}", self.prefix, prefix);

        let response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&full_prefix)
            .send()
            .await
            .context("Failed to list S3 objects")?;

        let keys = response
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|k| k.to_string()))
            .collect();

        Ok(keys)
    }
}

#[cfg(feature = "s3")]
#[async_trait]
impl Storage for S3Storage {
    async fn record_fetch(&self, job: &CrawlJob, result: &FetchResult) -> Result<()> {
        let fragment = url_to_fragment(&job.normalized_url);

        // Store body
        let body_key = self.body_key(&fragment);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&body_key)
            .body(ByteStream::from(result.body.clone()))
            .content_type(
                result
                    .content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
            )
            .send()
            .await
            .context("Failed to store body in S3")?;

        // Store metadata
        let metadata = serde_json::json!({
            "url": job.url,
            "final_url": result.final_url,
            "status": result.status,
            "content_type": result.content_type,
            "depth": job.depth,
            "body_key": body_key,
            "fetched_at_ms": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        });

        let metadata_json = serde_json::to_vec(&metadata)?;
        let metadata_key = self.metadata_key(&fragment);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .body(ByteStream::from(metadata_json))
            .content_type("application/json")
            .send()
            .await
            .context("Failed to store metadata in S3")?;

        Ok(())
    }
}

#[cfg(not(feature = "s3"))]
pub struct S3Storage;

#[cfg(not(feature = "s3"))]
impl S3Storage {
    pub async fn new(_bucket: String, _prefix: Option<String>) -> anyhow::Result<Self> {
        anyhow::bail!("S3 storage not enabled. Compile with 's3' feature.")
    }

    pub async fn new_with_endpoint(
        _bucket: String,
        _prefix: Option<String>,
        _endpoint_url: String,
    ) -> anyhow::Result<Self> {
        anyhow::bail!("S3 storage not enabled. Compile with 's3' feature.")
    }
}

#[cfg(all(test, feature = "s3"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires S3/MinIO setup
    async fn s3_storage_basic() {
        let storage = S3Storage::new("test-bucket".to_string(), Some("test/".to_string()))
            .await
            .unwrap();

        storage.verify_bucket().await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires MinIO setup
    async fn minio_storage() {
        let storage = S3Storage::new_with_endpoint(
            "test-bucket".to_string(),
            Some("test/".to_string()),
            "http://localhost:9000".to_string(),
        )
        .await
        .unwrap();

        storage.verify_bucket().await.unwrap();
    }
}
