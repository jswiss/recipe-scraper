use reqwest::Client;
use std::time::Duration;

use crate::url_ingestion::models::{FetchError, FetchSuccess, NormalizedUrl};

/// HTTP client configuration constants
const TIMEOUT_SECONDS: u64 = 30;
const MAX_REDIRECTS: usize = 5;
const MAX_SIZE_BYTES: usize = 10_485_760; // 10MB
const USER_AGENT: &str = "RecipeScraper/1.0";

/// Creates a configured HTTP client for URL fetching.
pub fn create_client() -> Result<Client, FetchError> {
    Client::builder()
        .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| FetchError::Network {
            message: format!("Failed to create HTTP client: {}", e),
            url: String::new(),
            details: Some(e.to_string()),
        })
}

/// Fetches HTML content from a URL.
///
/// Returns FetchSuccess with HTML content or FetchError on failure.
pub async fn fetch(
    client: &Client,
    normalized_url: &NormalizedUrl,
) -> Result<FetchSuccess, FetchError> {
    let url_string = normalized_url.to_url_string();

    // Make the GET request
    let response = client
        .get(&url_string)
        .send()
        .await
        .map_err(|e| map_reqwest_error(e, &url_string))?;

    // Check HTTP status
    let status = response.status();
    if status.is_client_error() {
        return Err(FetchError::Http {
            message: format_http_error(status.as_u16()),
            url: url_string,
            status_code: status.as_u16(),
        });
    }
    if status.is_server_error() {
        return Err(FetchError::Http {
            message: format_http_error(status.as_u16()),
            url: url_string,
            status_code: status.as_u16(),
        });
    }

    // Check Content-Type
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.starts_with("text/html") {
        return Err(FetchError::ContentType {
            message: format!("Expected HTML but received {}", content_type),
            url: url_string,
            content_type,
        });
    }

    // Get final URL (after redirects)
    let final_url = response.url().to_string();
    let final_url = if final_url != url_string {
        Some(final_url)
    } else {
        None
    };

    // Read body with size limit
    let html = read_body_with_limit(response, &url_string).await?;

    Ok(FetchSuccess {
        url: normalized_url.clone(),
        html,
        status_code: status.as_u16(),
        content_type,
        final_url,
    })
}

/// Reads the response body with a size limit.
async fn read_body_with_limit(
    response: reqwest::Response,
    url: &str,
) -> Result<String, FetchError> {
    // Check Content-Length header if available
    if let Some(content_length) = response.content_length() {
        if content_length as usize > MAX_SIZE_BYTES {
            return Err(FetchError::Size {
                message: format!(
                    "Response too large: {} bytes (max {} bytes)",
                    content_length, MAX_SIZE_BYTES
                ),
                url: url.to_string(),
                max_bytes: MAX_SIZE_BYTES,
            });
        }
    }

    // Stream body and enforce size limit
    let bytes = response.bytes().await.map_err(|e| FetchError::Network {
        message: format!("Failed to read response body: {}", e),
        url: url.to_string(),
        details: Some(e.to_string()),
    })?;

    if bytes.len() > MAX_SIZE_BYTES {
        return Err(FetchError::Size {
            message: format!(
                "Response too large: {} bytes (max {} bytes)",
                bytes.len(),
                MAX_SIZE_BYTES
            ),
            url: url.to_string(),
            max_bytes: MAX_SIZE_BYTES,
        });
    }

    // Convert to UTF-8 string
    String::from_utf8(bytes.to_vec()).map_err(|e| FetchError::ContentType {
        message: format!("Response is not valid UTF-8: {}", e),
        url: url.to_string(),
        content_type: "text/html".to_string(),
    })
}

/// Maps reqwest errors to FetchError.
fn map_reqwest_error(err: reqwest::Error, url: &str) -> FetchError {
    if err.is_timeout() {
        return FetchError::Network {
            message: "Request timed out after 30 seconds".to_string(),
            url: url.to_string(),
            details: None,
        };
    }

    if err.is_connect() {
        return FetchError::Network {
            message: "Connection failed".to_string(),
            url: url.to_string(),
            details: Some(err.to_string()),
        };
    }

    // Check for DNS resolution errors
    let err_string = err.to_string().to_lowercase();
    if err_string.contains("dns")
        || err_string.contains("resolve")
        || err_string.contains("no such host")
    {
        return FetchError::Network {
            message: "DNS resolution failed".to_string(),
            url: url.to_string(),
            details: Some(err.to_string()),
        };
    }

    FetchError::Network {
        message: format!("Network error: {}", err),
        url: url.to_string(),
        details: Some(err.to_string()),
    }
}

/// Formats HTTP status code into a human-readable message.
fn format_http_error(status_code: u16) -> String {
    match status_code {
        400 => "Bad request (400)".to_string(),
        401 => "Unauthorized (401)".to_string(),
        403 => "Forbidden (403)".to_string(),
        404 => "Page not found (404)".to_string(),
        405 => "Method not allowed (405)".to_string(),
        408 => "Request timeout (408)".to_string(),
        410 => "Gone (410)".to_string(),
        429 => "Too many requests (429)".to_string(),
        500 => "Internal server error (500)".to_string(),
        502 => "Bad gateway (502)".to_string(),
        503 => "Service unavailable (503)".to_string(),
        504 => "Gateway timeout (504)".to_string(),
        _ => format!("HTTP error ({})", status_code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_http_error_404() {
        assert_eq!(format_http_error(404), "Page not found (404)");
    }

    #[test]
    fn test_format_http_error_500() {
        assert_eq!(format_http_error(500), "Internal server error (500)");
    }

    #[test]
    fn test_format_http_error_unknown() {
        assert_eq!(format_http_error(418), "HTTP error (418)");
    }
}
