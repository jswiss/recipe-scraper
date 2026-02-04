use reqwest::Client;
use tauri::State;

use crate::url_ingestion::fetcher::{create_client, fetch};
use crate::url_ingestion::models::{FetchError, FetchSuccess, NormalizedUrl};
use crate::url_ingestion::normalizer::normalize;
use crate::url_ingestion::validator::validate;

/// Shared HTTP client state for Tauri commands.
pub struct HttpClient(pub Client);

/// Validates, normalizes, and fetches a URL.
///
/// This is the main Tauri command for URL ingestion.
#[tauri::command]
pub async fn ingest_url(
    url: String,
    client: State<'_, HttpClient>,
) -> Result<FetchSuccess, FetchError> {
    // Validate the URL
    let parsed = validate(&url)?;

    // Normalize the URL
    let normalized = normalize(&parsed);

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
