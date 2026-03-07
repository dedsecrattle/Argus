#[derive(Debug, Clone)]
pub struct CrawlJob {
    pub url: String,
    pub normalized_url: String,
    pub host: String,
    pub depth: u16,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: bytes::Bytes,
}

#[derive(Debug, Clone)]
pub struct ExtractedLink {
    pub from_url: String,
    pub to_url: String,
}
