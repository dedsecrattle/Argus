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
