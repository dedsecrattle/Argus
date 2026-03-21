use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Allow(String),
    Disallow(String),
}

#[derive(Debug, Clone)]
pub struct RobotsTxt {
    rules: Vec<Rule>,
    crawl_delay: Option<Duration>,
}

impl RobotsTxt {
    pub fn parse(content: &str, user_agent: &str) -> Self {
        let mut rules = Vec::new();
        let mut crawl_delay = None;
        let mut in_matching_section = false;
        let mut in_any_section = false;

        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let line = if let Some(pos) = line.find('#') {
                &line[..pos].trim()
            } else {
                line
            };

            let (key, value) = match line.split_once(':') {
                Some((k, v)) => (k.trim().to_lowercase(), v.trim()),
                None => continue,
            };

            match key.as_str() {
                "user-agent" => {
                    if in_matching_section {
                        break;
                    }
                    in_any_section = true;
                    let agent_pattern = value.to_lowercase();
                    in_matching_section = agent_pattern == "*" 
                        || user_agent.to_lowercase().contains(&agent_pattern)
                        || agent_pattern.contains(&user_agent.to_lowercase());
                }
                "allow" if in_matching_section => {
                    rules.push(Rule::Allow(value.to_string()));
                }
                "disallow" if in_matching_section => {
                    rules.push(Rule::Disallow(value.to_string()));
                }
                "crawl-delay" if in_matching_section => {
                    if let Ok(seconds) = value.parse::<f64>() {
                        crawl_delay = Some(Duration::from_secs_f64(seconds));
                    }
                }
                _ => {}
            }
        }

        if !in_any_section {
            rules.push(Rule::Allow("/".to_string()));
        }

        Self { rules, crawl_delay }
    }

    pub fn is_allowed(&self, path: &str) -> bool {
        if self.rules.is_empty() {
            return true;
        }

        let mut allowed = true;
        let mut best_match_len = 0;

        for rule in &self.rules {
            let (is_allow, pattern) = match rule {
                Rule::Allow(p) => (true, p),
                Rule::Disallow(p) => (false, p),
            };

            if pattern.is_empty() {
                continue;
            }

            if self.matches_pattern(path, pattern) {
                let pattern_len = pattern.len();
                if pattern_len > best_match_len {
                    best_match_len = pattern_len;
                    allowed = is_allow;
                }
            }
        }

        allowed
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern == "/" {
            return true;
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }

        if pattern.ends_with('$') {
            let prefix = &pattern[..pattern.len() - 1];
            return path == prefix;
        }

        path.starts_with(pattern)
    }

    pub fn crawl_delay(&self) -> Option<Duration> {
        self.crawl_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_robots_txt() {
        let robots = RobotsTxt::parse("", "TestBot");
        assert!(robots.is_allowed("/"));
        assert!(robots.is_allowed("/anything"));
    }

    #[test]
    fn parse_wildcard_user_agent() {
        let content = r#"
User-agent: *
Disallow: /admin/
Allow: /admin/public
"#;
        let robots = RobotsTxt::parse(content, "TestBot");
        assert!(!robots.is_allowed("/admin/"));
        assert!(!robots.is_allowed("/admin/secret"));
        assert!(robots.is_allowed("/admin/public"));
        assert!(robots.is_allowed("/"));
    }

    #[test]
    fn parse_specific_user_agent() {
        let content = r#"
User-agent: BadBot
Disallow: /

User-agent: GoodBot
Disallow: /private/
"#;
        let robots = RobotsTxt::parse(content, "GoodBot");
        assert!(!robots.is_allowed("/private/"));
        assert!(robots.is_allowed("/public/"));
    }

    #[test]
    fn parse_crawl_delay() {
        let content = r#"
User-agent: *
Crawl-delay: 2.5
"#;
        let robots = RobotsTxt::parse(content, "TestBot");
        assert_eq!(robots.crawl_delay(), Some(Duration::from_secs_f64(2.5)));
    }

    #[test]
    fn pattern_matching_wildcard() {
        let content = r#"
User-agent: *
Disallow: /temp*
"#;
        let robots = RobotsTxt::parse(content, "TestBot");
        assert!(!robots.is_allowed("/temp"));
        assert!(!robots.is_allowed("/temporary"));
        assert!(robots.is_allowed("/other"));
    }

    #[test]
    fn pattern_matching_end_anchor() {
        let content = r#"
User-agent: *
Disallow: /file.html$
"#;
        let robots = RobotsTxt::parse(content, "TestBot");
        assert!(!robots.is_allowed("/file.html"));
        assert!(robots.is_allowed("/file.html?query=1"));
    }

    #[test]
    fn longest_match_wins() {
        let content = r#"
User-agent: *
Disallow: /admin/
Allow: /admin/public/
"#;
        let robots = RobotsTxt::parse(content, "TestBot");
        assert!(!robots.is_allowed("/admin/"));
        assert!(!robots.is_allowed("/admin/secret"));
        assert!(robots.is_allowed("/admin/public/"));
        assert!(robots.is_allowed("/admin/public/page"));
    }

    #[test]
    fn case_insensitive_user_agent() {
        let content = r#"
User-agent: googlebot
Disallow: /private/
"#;
        let robots = RobotsTxt::parse(content, "GoogleBot");
        assert!(!robots.is_allowed("/private/"));
    }

    #[test]
    fn comments_ignored() {
        let content = r#"
# This is a comment
User-agent: *
Disallow: /admin/ # inline comment
"#;
        let robots = RobotsTxt::parse(content, "TestBot");
        assert!(!robots.is_allowed("/admin/"));
    }
}
