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
