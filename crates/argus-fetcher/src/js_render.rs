#[cfg(feature = "js-render")]
use anyhow::{Context, Result};
#[cfg(feature = "js-render")]
use headless_chrome::{Browser, LaunchOptions};
#[cfg(feature = "js-render")]
use std::time::Duration;

#[cfg(feature = "js-render")]
#[derive(Clone)]
pub struct JsRenderer {
    headless: bool,
    timeout: Duration,
}

#[cfg(feature = "js-render")]
impl JsRenderer {
    pub fn new() -> Self {
        Self {
            headless: true,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn render(&self, url: &str) -> Result<String> {
        let headless = self.headless;
        let timeout = self.timeout;
        let url = url.to_string();

        tokio::task::spawn_blocking(move || {
            let options = LaunchOptions {
                headless,
                ..Default::default()
            };

            let browser = Browser::new(options).context("failed to launch browser")?;
            let tab = browser.new_tab().context("failed to create tab")?;

            tab.navigate_to(&url).context("failed to navigate to URL")?;

            tab.wait_for_element_with_custom_timeout("body", timeout)
                .context("timeout waiting for body element")?;

            std::thread::sleep(Duration::from_millis(500));

            let content = tab.get_content().context("failed to get page content")?;

            Ok(content)
        })
        .await
        .context("render task panicked")?
    }
}

#[cfg(feature = "js-render")]
impl Default for JsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "js-render"))]
pub struct JsRenderer;

#[cfg(not(feature = "js-render"))]
impl JsRenderer {
    pub fn new() -> Self {
        Self
    }

    pub async fn render(&self, _url: &str) -> anyhow::Result<String> {
        anyhow::bail!("JavaScript rendering not enabled. Compile with 'js-render' feature.")
    }
}

#[cfg(not(feature = "js-render"))]
impl Default for JsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "js-render"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn render_simple_page() {
        let renderer = JsRenderer::new();
        let result = renderer.render("https://example.com").await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Example Domain"));
    }

    #[tokio::test]
    async fn render_with_custom_timeout() {
        let renderer = JsRenderer::new().with_timeout(Duration::from_secs(10));
        let result = renderer.render("https://example.com").await;
        assert!(result.is_ok());
    }
}
