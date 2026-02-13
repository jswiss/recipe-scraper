use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Categories of errors that can occur during URL ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// URL syntax or protocol errors
    Validation,
    /// DNS, timeout, connection failures
    Network,
    /// HTTP 4xx/5xx responses
    Http,
    /// Non-HTML responses
    ContentType,
    /// Response exceeds 10MB limit
    Size,
    /// Scraping disallowed by robots.txt
    RobotsDisallowed,
}

/// A validated and normalized URL ready for fetching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedUrl {
    /// Protocol: "http" or "https"
    pub scheme: String,
    /// Domain in ASCII (Punycode if IDN)
    pub host: String,
    /// Port number, None if default (80 for http, 443 for https)
    pub port: Option<u16>,
    /// URL path, starts with "/"
    pub path: String,
    /// Query string without leading "?"
    pub query: Option<String>,
    /// Fragment without leading "#"
    pub fragment: Option<String>,
}

impl NormalizedUrl {
    /// Reconstructs the full URL string from components.
    pub fn to_url_string(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);

        if let Some(port) = self.port {
            url.push_str(&format!(":{}", port));
        }

        url.push_str(&self.path);

        if let Some(ref query) = self.query {
            url.push('?');
            url.push_str(query);
        }

        if let Some(ref fragment) = self.fragment {
            url.push('#');
            url.push_str(fragment);
        }

        url
    }
}

impl fmt::Display for NormalizedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_url_string())
    }
}

/// Successful fetch result containing HTML content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSuccess {
    /// The normalized URL fetched
    pub url: NormalizedUrl,
    /// HTML content (UTF-8 decoded)
    pub html: String,
    /// HTTP status code (200-299)
    pub status_code: u16,
    /// Content-Type header value
    pub content_type: String,
    /// Final URL if redirected, None if no redirect
    pub final_url: Option<String>,
}

/// Structured error for failed operations.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum FetchError {
    /// Invalid URL syntax or protocol
    #[error("{message}")]
    Validation { message: String, url: String },

    /// DNS, timeout, or connection errors
    #[error("{message}")]
    Network {
        message: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// HTTP 4xx/5xx responses
    #[error("{message}")]
    Http {
        message: String,
        url: String,
        status_code: u16,
    },

    /// Non-HTML response
    #[error("{message}")]
    ContentType {
        message: String,
        url: String,
        content_type: String,
    },

    /// Response exceeds size limit
    #[error("{message}")]
    Size {
        message: String,
        url: String,
        max_bytes: usize,
    },

    /// Scraping disallowed by robots.txt
    #[error("{message}")]
    RobotsDisallowed {
        message: String,
        url: String,
        reason: String,
    },
}

impl FetchError {
    /// Returns the error type category.
    pub fn error_type(&self) -> ErrorType {
        match self {
            FetchError::Validation { .. } => ErrorType::Validation,
            FetchError::Network { .. } => ErrorType::Network,
            FetchError::Http { .. } => ErrorType::Http,
            FetchError::ContentType { .. } => ErrorType::ContentType,
            FetchError::Size { .. } => ErrorType::Size,
            FetchError::RobotsDisallowed { .. } => ErrorType::RobotsDisallowed,
        }
    }

    /// Returns the original URL associated with the error.
    pub fn url(&self) -> &str {
        match self {
            FetchError::Validation { url, .. }
            | FetchError::Network { url, .. }
            | FetchError::Http { url, .. }
            | FetchError::ContentType { url, .. }
            | FetchError::Size { url, .. }
            | FetchError::RobotsDisallowed { url, .. } => url,
        }
    }
}

/// Result type for URL ingestion operations.
pub type FetchResult = Result<FetchSuccess, FetchError>;
