use async_trait::async_trait;

#[async_trait]
pub trait SeenSet: Send + Sync {
    /// Returns true if the URL was newly inserted, false if it was already seen.
    async fn insert_if_new(&self, url: String) -> bool;
}
