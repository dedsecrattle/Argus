use argus_common::ExtractedLink;
use scraper::{Html, Selector};
use url::Url;

#[derive(Debug, Clone)]
pub struct PageMetadata {
    pub canonical_url: Option<String>,
    pub alternate_urls: Vec<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub fn extract_links(base_url: &str, body: &[u8]) -> Vec<ExtractedLink> {
    let html = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let base = match Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };

    let document = Html::parse_document(html);
    let mut links = Vec::new();

    if let Ok(selector) = Selector::parse("a[href]") {
        for el in document.select(&selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(url) = base.join(href) {
                    links.push(ExtractedLink {
                        from_url: base_url.to_string(),
                        to_url: url.to_string(),
                    });
                }
            }
        }
    }

    if let Ok(selector) = Selector::parse("link[rel='alternate'][href]") {
        for el in document.select(&selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(url) = base.join(href) {
                    links.push(ExtractedLink {
                        from_url: base_url.to_string(),
                        to_url: url.to_string(),
                    });
                }
            }
        }
    }

    links
}

pub fn extract_metadata(body: &[u8]) -> PageMetadata {
    let html = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => return PageMetadata::default(),
    };

    let document = Html::parse_document(html);
    let mut metadata = PageMetadata::default();

    if let Ok(selector) = Selector::parse("link[rel='canonical'][href]") {
        if let Some(el) = document.select(&selector).next() {
            metadata.canonical_url = el.value().attr("href").map(|s| s.to_string());
        }
    }

    if let Ok(selector) = Selector::parse("link[rel='alternate'][hreflang][href]") {
        for el in document.select(&selector) {
            if let Some(href) = el.value().attr("href") {
                metadata.alternate_urls.push(href.to_string());
            }
        }
    }

    if let Ok(selector) = Selector::parse("title") {
        if let Some(el) = document.select(&selector).next() {
            metadata.title = Some(el.text().collect::<String>().trim().to_string());
        }
    }

    if let Ok(selector) = Selector::parse("meta[name='description'][content]") {
        if let Some(el) = document.select(&selector).next() {
            metadata.description = el.value().attr("content").map(|s| s.to_string());
        }
    }

    metadata
}

impl Default for PageMetadata {
    fn default() -> Self {
        Self {
            canonical_url: None,
            alternate_urls: Vec::new(),
            title: None,
            description: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_absolute_link() {
        let html = b"<a href=\"https://example.com/other\">link</a>";
        let links = extract_links("https://example.com/page", html);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].from_url, "https://example.com/page");
        assert_eq!(links[0].to_url, "https://example.com/other");
    }

    #[test]
    fn resolves_relative_link() {
        let html = b"<a href=\"/about\">about</a>";
        let links = extract_links("https://example.com/", html);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].to_url, "https://example.com/about");
    }

    #[test]
    fn returns_empty_for_invalid_utf8() {
        let body = b"\xff\xfe";
        let links = extract_links("https://example.com/", body);
        assert!(links.is_empty());
    }

    #[test]
    fn returns_empty_for_no_links() {
        let html = b"<p>no links here</p>";
        let links = extract_links("https://example.com/", html);
        assert!(links.is_empty());
    }

    #[test]
    fn extracts_canonical_url() {
        let html = b"<link rel=\"canonical\" href=\"https://example.com/canonical\">";
        let metadata = extract_metadata(html);
        assert_eq!(
            metadata.canonical_url,
            Some("https://example.com/canonical".to_string())
        );
    }

    #[test]
    fn extracts_alternate_urls() {
        let html = b"<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/es\">\
                     <link rel=\"alternate\" hreflang=\"fr\" href=\"https://example.com/fr\">";
        let metadata = extract_metadata(html);
        assert_eq!(metadata.alternate_urls.len(), 2);
        assert!(metadata
            .alternate_urls
            .contains(&"https://example.com/es".to_string()));
        assert!(metadata
            .alternate_urls
            .contains(&"https://example.com/fr".to_string()));
    }

    #[test]
    fn extracts_title_and_description() {
        let html = b"<title>Page Title</title>\
                     <meta name=\"description\" content=\"Page description\">";
        let metadata = extract_metadata(html);
        assert_eq!(metadata.title, Some("Page Title".to_string()));
        assert_eq!(metadata.description, Some("Page description".to_string()));
    }

    #[test]
    fn extracts_alternate_links() {
        let html = b"<a href=\"/page1\">Link 1</a>\
                     <link rel=\"alternate\" href=\"/page2\">";
        let links = extract_links("https://example.com/", html);
        assert_eq!(links.len(), 2);
        assert!(links
            .iter()
            .any(|l| l.to_url == "https://example.com/page1"));
        assert!(links
            .iter()
            .any(|l| l.to_url == "https://example.com/page2"));
    }
}
