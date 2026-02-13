/// Extracts the `Crawl-delay` value from raw robots.txt content
/// for the most relevant user-agent group.
///
/// Checks the specific user-agent group first, then falls back to `*`.
/// Returns `None` if no `Crawl-delay` directive is found.
pub fn parse_crawl_delay(raw_content: &str, user_agent: &str) -> Option<f64> {
    let mut in_specific_group = false;
    let mut in_wildcard_group = false;
    let mut specific_delay: Option<f64> = None;
    let mut wildcard_delay: Option<f64> = None;

    let ua_lower = user_agent.to_lowercase();

    for line in raw_content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(rest) = trimmed
            .strip_prefix("User-agent:")
            .or_else(|| trimmed.strip_prefix("user-agent:"))
        {
            let agent = rest.trim().to_lowercase();
            in_specific_group = agent == ua_lower;
            in_wildcard_group = agent == "*";
            continue;
        }

        if let Some(rest) = trimmed
            .strip_prefix("Crawl-delay:")
            .or_else(|| trimmed.strip_prefix("crawl-delay:"))
        {
            if let Ok(val) = rest.trim().parse::<f64>() {
                if val >= 0.0 {
                    if in_specific_group {
                        specific_delay = Some(val);
                    } else if in_wildcard_group {
                        wildcard_delay = Some(val);
                    }
                }
            }
        }
    }

    // Specific agent group takes priority over wildcard
    specific_delay.or(wildcard_delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_agent_delay() {
        let content = "User-agent: RecipeScraper\nCrawl-delay: 10\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), Some(10.0));
    }

    #[test]
    fn test_wildcard_delay() {
        let content = "User-agent: *\nCrawl-delay: 5\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), Some(5.0));
    }

    #[test]
    fn test_specific_overrides_wildcard() {
        let content =
            "User-agent: *\nCrawl-delay: 5\n\nUser-agent: RecipeScraper\nCrawl-delay: 2\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), Some(2.0));
    }

    #[test]
    fn test_no_delay() {
        let content = "User-agent: *\nDisallow: /private\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), None);
    }

    #[test]
    fn test_fractional_delay() {
        let content = "User-agent: *\nCrawl-delay: 0.5\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), Some(0.5));
    }

    #[test]
    fn test_case_insensitive_agent() {
        let content = "User-agent: recipescraper\nCrawl-delay: 3\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), Some(3.0));
    }

    #[test]
    fn test_invalid_delay_ignored() {
        let content = "User-agent: *\nCrawl-delay: abc\n";
        assert_eq!(parse_crawl_delay(content, "RecipeScraper"), None);
    }
}
