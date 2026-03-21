use url::Url;

#[derive(Debug, Clone)]
pub struct SitemapUrl {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum SitemapEntry {
    Url(SitemapUrl),
    Index(String),
}

pub fn parse_sitemap(content: &str) -> Vec<SitemapEntry> {
    let mut entries = Vec::new();
    let mut current_url: Option<SitemapUrl> = None;
    let mut in_url = false;
    let mut in_sitemap = false;
    let mut current_tag = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("<url>") {
            in_url = true;
            current_url = Some(SitemapUrl {
                loc: String::new(),
                lastmod: None,
                changefreq: None,
                priority: None,
            });
        } else if line.starts_with("</url>") {
            in_url = false;
            if let Some(url) = current_url.take() {
                if !url.loc.is_empty() {
                    entries.push(SitemapEntry::Url(url));
                }
            }
        } else if line.starts_with("<sitemap>") {
            in_sitemap = true;
        } else if line.starts_with("</sitemap>") {
            in_sitemap = false;
        } else if in_url || in_sitemap {
            if let Some(tag_start) = line.find('<') {
                if let Some(tag_end) = line.find('>') {
                    let tag = &line[tag_start + 1..tag_end];
                    if !tag.starts_with('/') {
                        current_tag = tag.to_string();
                        let content_start = tag_end + 1;
                        if let Some(close_start) = line[content_start..].find('<') {
                            current_content = line[content_start..content_start + close_start]
                                .trim()
                                .to_string();

                            if in_url {
                                if let Some(ref mut url) = current_url {
                                    match current_tag.as_str() {
                                        "loc" => url.loc = current_content.clone(),
                                        "lastmod" => url.lastmod = Some(current_content.clone()),
                                        "changefreq" => {
                                            url.changefreq = Some(current_content.clone())
                                        }
                                        "priority" => {
                                            url.priority = current_content.parse().ok()
                                        }
                                        _ => {}
                                    }
                                }
                            } else if in_sitemap && current_tag == "loc" {
                                entries.push(SitemapEntry::Index(current_content.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    entries
}

pub fn discover_sitemap_urls(base_url: &str) -> Vec<String> {
    let base = match Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };

    let origin = format!(
        "{}://{}{}",
        base.scheme(),
        base.host_str().unwrap_or(""),
        if let Some(port) = base.port() {
            format!(":{}", port)
        } else {
            String::new()
        }
    );

    vec![
        format!("{}/sitemap.xml", origin),
        format!("{}/sitemap_index.xml", origin),
        format!("{}/sitemap-index.xml", origin),
        format!("{}/robots.txt", origin),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_sitemap() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/page1</loc>
    <lastmod>2024-01-01</lastmod>
    <changefreq>daily</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>https://example.com/page2</loc>
  </url>
</urlset>"#;

        let entries = parse_sitemap(xml);
        assert_eq!(entries.len(), 2);

        match &entries[0] {
            SitemapEntry::Url(url) => {
                assert_eq!(url.loc, "https://example.com/page1");
                assert_eq!(url.lastmod, Some("2024-01-01".to_string()));
                assert_eq!(url.changefreq, Some("daily".to_string()));
                assert_eq!(url.priority, Some(0.8));
            }
            _ => panic!("Expected URL entry"),
        }

        match &entries[1] {
            SitemapEntry::Url(url) => {
                assert_eq!(url.loc, "https://example.com/page2");
                assert_eq!(url.lastmod, None);
            }
            _ => panic!("Expected URL entry"),
        }
    }

    #[test]
    fn parse_sitemap_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://example.com/sitemap1.xml</loc>
  </sitemap>
  <sitemap>
    <loc>https://example.com/sitemap2.xml</loc>
  </sitemap>
</sitemapindex>"#;

        let entries = parse_sitemap(xml);
        assert_eq!(entries.len(), 2);

        match &entries[0] {
            SitemapEntry::Index(url) => {
                assert_eq!(url, "https://example.com/sitemap1.xml");
            }
            _ => panic!("Expected Index entry"),
        }
    }

    #[test]
    fn discover_sitemap_urls_generates_common_paths() {
        let urls = discover_sitemap_urls("https://example.com/page");
        assert!(urls.contains(&"https://example.com/sitemap.xml".to_string()));
        assert!(urls.contains(&"https://example.com/sitemap_index.xml".to_string()));
        assert!(urls.contains(&"https://example.com/robots.txt".to_string()));
    }

    #[test]
    fn parse_empty_sitemap() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
</urlset>"#;

        let entries = parse_sitemap(xml);
        assert_eq!(entries.len(), 0);
    }
}
