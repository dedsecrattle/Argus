use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Html,
    Text,
    Json,
    Xml,
    Pdf,
    Image,
    Video,
    Audio,
    Binary,
    Unknown,
}

impl ContentType {
    pub fn from_mime(mime: &str) -> Self {
        let mime_lower = mime.to_lowercase();
        let base = mime_lower.split(';').next().unwrap_or("").trim();

        match base {
            "text/html" | "application/xhtml+xml" => Self::Html,
            "text/plain" => Self::Text,
            "application/json" | "application/ld+json" => Self::Json,
            "text/xml" | "application/xml" | "application/rss+xml" | "application/atom+xml" => {
                Self::Xml
            }
            "application/pdf" => Self::Pdf,
            mime if mime.starts_with("image/") => Self::Image,
            mime if mime.starts_with("video/") => Self::Video,
            mime if mime.starts_with("audio/") => Self::Audio,
            mime if mime.starts_with("text/") => Self::Text,
            _ => Self::Binary,
        }
    }

    pub fn is_text_based(&self) -> bool {
        matches!(
            self,
            Self::Html | Self::Text | Self::Json | Self::Xml
        )
    }

    pub fn is_crawlable(&self) -> bool {
        matches!(self, Self::Html | Self::Xml)
    }
}

#[derive(Debug, Clone)]
pub struct ContentLimits {
    pub max_html_size: usize,
    pub max_text_size: usize,
    pub max_binary_size: usize,
    pub allow_binary: bool,
}

impl Default for ContentLimits {
    fn default() -> Self {
        Self {
            max_html_size: 10 * 1024 * 1024,
            max_text_size: 5 * 1024 * 1024,
            max_binary_size: 50 * 1024 * 1024,
            allow_binary: false,
        }
    }
}

impl ContentLimits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_html_size(mut self, size: usize) -> Self {
        self.max_html_size = size;
        self
    }

    pub fn with_max_text_size(mut self, size: usize) -> Self {
        self.max_text_size = size;
        self
    }

    pub fn with_max_binary_size(mut self, size: usize) -> Self {
        self.max_binary_size = size;
        self
    }

    pub fn with_allow_binary(mut self, allow: bool) -> Self {
        self.allow_binary = allow;
        self
    }

    pub fn check_size(&self, content_type: ContentType, size: usize) -> Result<()> {
        let max_size = match content_type {
            ContentType::Html => self.max_html_size,
            ContentType::Text | ContentType::Json | ContentType::Xml => self.max_text_size,
            _ => {
                if !self.allow_binary {
                    anyhow::bail!("binary content not allowed");
                }
                self.max_binary_size
            }
        };

        if size > max_size {
            anyhow::bail!(
                "content size {} exceeds limit {} for {:?}",
                size,
                max_size,
                content_type
            );
        }

        Ok(())
    }
}

pub fn detect_encoding(body: &[u8]) -> Option<&'static str> {
    if body.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Some("utf-8");
    }

    if body.starts_with(&[0xFE, 0xFF]) {
        return Some("utf-16be");
    }

    if body.starts_with(&[0xFF, 0xFE]) {
        return Some("utf-16le");
    }

    if std::str::from_utf8(body).is_ok() {
        return Some("utf-8");
    }

    None
}

pub fn extract_text_content(body: &[u8], content_type: ContentType) -> Result<String> {
    if !content_type.is_text_based() {
        anyhow::bail!("cannot extract text from non-text content");
    }

    let encoding = detect_encoding(body).unwrap_or("utf-8");

    if encoding == "utf-8" {
        String::from_utf8(body.to_vec()).context("invalid utf-8")
    } else {
        anyhow::bail!("unsupported encoding: {}", encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_from_mime() {
        assert_eq!(ContentType::from_mime("text/html"), ContentType::Html);
        assert_eq!(
            ContentType::from_mime("text/html; charset=utf-8"),
            ContentType::Html
        );
        assert_eq!(ContentType::from_mime("application/json"), ContentType::Json);
        assert_eq!(ContentType::from_mime("application/xml"), ContentType::Xml);
        assert_eq!(ContentType::from_mime("application/pdf"), ContentType::Pdf);
        assert_eq!(ContentType::from_mime("image/png"), ContentType::Image);
        assert_eq!(ContentType::from_mime("video/mp4"), ContentType::Video);
        assert_eq!(
            ContentType::from_mime("application/octet-stream"),
            ContentType::Binary
        );
    }

    #[test]
    fn text_based_content() {
        assert!(ContentType::Html.is_text_based());
        assert!(ContentType::Text.is_text_based());
        assert!(ContentType::Json.is_text_based());
        assert!(ContentType::Xml.is_text_based());
        assert!(!ContentType::Pdf.is_text_based());
        assert!(!ContentType::Image.is_text_based());
    }

    #[test]
    fn crawlable_content() {
        assert!(ContentType::Html.is_crawlable());
        assert!(ContentType::Xml.is_crawlable());
        assert!(!ContentType::Text.is_crawlable());
        assert!(!ContentType::Json.is_crawlable());
        assert!(!ContentType::Pdf.is_crawlable());
    }

    #[test]
    fn content_limits_default() {
        let limits = ContentLimits::default();
        assert_eq!(limits.max_html_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_text_size, 5 * 1024 * 1024);
        assert!(!limits.allow_binary);
    }

    #[test]
    fn content_limits_check_size() {
        let limits = ContentLimits::default();

        assert!(limits
            .check_size(ContentType::Html, 1024 * 1024)
            .is_ok());

        assert!(limits
            .check_size(ContentType::Html, 20 * 1024 * 1024)
            .is_err());

        assert!(limits.check_size(ContentType::Image, 1024).is_err());

        let limits_with_binary = limits.with_allow_binary(true);
        assert!(limits_with_binary
            .check_size(ContentType::Image, 1024)
            .is_ok());
    }

    #[test]
    fn detect_utf8_encoding() {
        let text = "Hello, world!";
        assert_eq!(detect_encoding(text.as_bytes()), Some("utf-8"));
    }

    #[test]
    fn detect_utf8_bom() {
        let text = b"\xEF\xBB\xBFHello";
        assert_eq!(detect_encoding(text), Some("utf-8"));
    }

    #[test]
    fn extract_text_from_html() {
        let html = b"<html><body>Hello</body></html>";
        let text = extract_text_content(html, ContentType::Html).unwrap();
        assert_eq!(text, "<html><body>Hello</body></html>");
    }

    #[test]
    fn extract_text_fails_for_binary() {
        let binary = b"\xFF\xFE\x00\x01";
        let result = extract_text_content(binary, ContentType::Image);
        assert!(result.is_err());
    }
}
