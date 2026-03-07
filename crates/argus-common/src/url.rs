use url::Url;

pub fn normalize_url(input: &str) -> Option<(String, String)> {
    let mut url = Url::parse(input).ok()?;
    url.set_fragment(None);

    let host = url.host_str()?.to_ascii_lowercase();

    if (url.scheme() == "http" && url.port() == Some(80))
        || (url.scheme() == "https" && url.port() == Some(443))
    {
        let _ = url.set_port(None);
    }

    Some((url.to_string(), host))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_https_and_strips_default_port() {
        let (url, host) = normalize_url("https://example.com:443/path").unwrap();
        assert_eq!(url, "https://example.com/path");
        assert_eq!(host, "example.com");
    }

    #[test]
    fn strips_fragment() {
        let (url, _) = normalize_url("https://example.com/page#section").unwrap();
        assert_eq!(url, "https://example.com/page");
    }

    #[test]
    fn rejects_invalid_url() {
        assert!(normalize_url("not a url").is_none());
    }

    #[test]
    fn rejects_mailto() {
        assert!(normalize_url("mailto:foo@bar.com").is_none());
    }
}
