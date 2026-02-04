//! URL Ingestion Module
//!
//! Provides URL validation, normalization, and HTML fetching functionality.
//!
//! # Example
//!
//! ```rust,ignore
//! use url_ingestion::{ingest_url, validate_url, FetchSuccess, FetchError};
//!
//! // Validate and fetch a URL
//! let result = ingest_url("https://example.com/recipe").await;
//! match result {
//!     Ok(success) => println!("Fetched {} bytes", success.html.len()),
//!     Err(error) => println!("Error: {}", error),
//! }
//!
//! // Validate only (no network request)
//! let normalized = validate_url("https://EXAMPLE.COM/Recipe/").await;
//! ```

mod commands;
mod fetcher;
pub mod models;
mod normalizer;
mod validator;

// Re-export Tauri commands
pub use commands::{create_http_client, ingest_url, validate_url, HttpClient};

// Re-export models for external use
pub use models::{ErrorType, FetchError, FetchResult, FetchSuccess, NormalizedUrl};

// Re-export internal functions for library use
pub use fetcher::create_client;
pub use normalizer::normalize;
pub use validator::validate;
