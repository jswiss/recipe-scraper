use reqwest::Client;
use tauri::State;

use crate::robots_compliance::checker::check_compliance;
use crate::storage::Database;
use crate::url_ingestion::fetcher::{create_client, fetch};
use crate::url_ingestion::models::{FetchError, FetchSuccess, NormalizedUrl};
use crate::url_ingestion::normalizer::normalize;
use crate::url_ingestion::validator::validate;

/// Shared HTTP client state for Tauri commands.
pub struct HttpClient(pub Client);

/// Validates, normalizes, and fetches a URL.
///
/// This is the main Tauri command for URL ingestion. It checks robots.txt
/// compliance before fetching and rejects disallowed URLs.
#[tauri::command]
pub async fn ingest_url(
    url: String,
    client: State<'_, HttpClient>,
    db: State<'_, Database>,
) -> Result<FetchSuccess, FetchError> {
    // Validate the URL
    let parsed = validate(&url)?;

    // Normalize the URL
    let normalized = normalize(&parsed);
    let url_string = normalized.to_url_string();

    // Check robots.txt compliance before fetching
    let decision = check_compliance(&client.0, &db, &url_string)
        .await
        .map_err(|e| FetchError::Network {
            message: format!("Robots.txt check failed: {e}"),
            url: url_string.clone(),
            details: None,
        })?;

    if !decision.allowed {
        return Err(FetchError::RobotsDisallowed {
            message: "Scraping disallowed by robots.txt".to_string(),
            url: url_string,
            reason: decision.reason,
        });
    }

    // Fetch the content
    fetch(&client.0, &normalized).await
}

/// Validates and normalizes a URL without fetching.
///
/// Useful for checking if a URL is valid before attempting to fetch.
#[tauri::command]
pub async fn validate_url(url: String) -> Result<NormalizedUrl, FetchError> {
    // Validate the URL
    let parsed = validate(&url)?;

    // Normalize the URL
    Ok(normalize(&parsed))
}

/// Creates a new HttpClient for use in Tauri state.
pub fn create_http_client() -> Result<HttpClient, FetchError> {
    create_client().map(HttpClient)
}

#[cfg(test)]
mod tests {
    use crate::url_ingestion::models::FetchError;
    use crate::url_ingestion::normalizer::normalize;
    use crate::url_ingestion::validator::validate;

    #[test]
    fn test_validate_url_valid_returns_normalized() {
        let parsed = validate("https://EXAMPLE.COM/path/").expect("URL should be valid");
        let normalized = normalize(&parsed);
        assert_eq!(
            normalized.host, "example.com",
            "Host should be lowercased after normalization"
        );
    }

    #[test]
    fn test_validate_url_invalid_returns_error() {
        let result = validate("not-a-url");
        assert!(result.is_err(), "Invalid URL should return an error");
        assert!(
            matches!(result.unwrap_err(), FetchError::Validation { .. }),
            "Error should be a Validation variant"
        );
    }
}
