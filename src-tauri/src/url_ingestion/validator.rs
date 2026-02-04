use url::Url;

use crate::url_ingestion::models::FetchError;

/// Validates a URL string and returns a parsed URL or validation error.
///
/// Validation rules:
/// - URL must be non-empty
/// - URL must be parseable as a valid URL
/// - Scheme must be http or https
pub fn validate(input: &str) -> Result<Url, FetchError> {
    // Check for empty or whitespace-only input
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(FetchError::Validation {
            message: "No URL provided".to_string(),
            url: input.to_string(),
        });
    }

    // Check for missing scheme (common user error)
    if !trimmed.contains("://") {
        return Err(FetchError::Validation {
            message: "Invalid URL: missing scheme (try adding http:// or https://)".to_string(),
            url: input.to_string(),
        });
    }

    // Parse the URL
    let parsed = Url::parse(trimmed).map_err(|e| FetchError::Validation {
        message: format!("Invalid URL: {}", e),
        url: input.to_string(),
    })?;

    // Validate protocol
    validate_protocol(&parsed, input)?;

    Ok(parsed)
}

/// Validates that the URL uses an allowed protocol (http or https).
fn validate_protocol(url: &Url, original: &str) -> Result<(), FetchError> {
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(FetchError::Validation {
            message: format!(
                "Invalid URL: scheme '{}' not allowed (only http and https)",
                scheme
            ),
            url: original.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        let result = validate("https://example.com/path");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_valid_http_url() {
        let result = validate("http://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_url() {
        let result = validate("");
        assert!(result.is_err());
        if let Err(FetchError::Validation { message, .. }) = result {
            assert_eq!(message, "No URL provided");
        }
    }

    #[test]
    fn test_whitespace_url() {
        let result = validate("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_scheme() {
        let result = validate("example.com");
        assert!(result.is_err());
        if let Err(FetchError::Validation { message, .. }) = result {
            assert!(message.contains("missing scheme"));
        }
    }

    #[test]
    fn test_invalid_scheme_ftp() {
        let result = validate("ftp://example.com");
        assert!(result.is_err());
        if let Err(FetchError::Validation { message, .. }) = result {
            assert!(message.contains("ftp"));
            assert!(message.contains("not allowed"));
        }
    }

    #[test]
    fn test_invalid_scheme_file() {
        let result = validate("file:///etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_scheme_mailto() {
        let result = validate("mailto:user@example.com");
        assert!(result.is_err());
    }
}
