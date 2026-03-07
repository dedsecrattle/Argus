use argus_common::ExtractedLink;
use scraper::{Html, Selector};
use url::Url;

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
    let selector = Selector::parse("a[href]").unwrap();

    document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| base.join(href).ok())
        .map(|u| ExtractedLink {
            from_url: base_url.to_string(),
            to_url: u.to_string(),
        })
        .collect()
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
}
