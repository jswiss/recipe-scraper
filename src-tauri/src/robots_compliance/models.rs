use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Whether the robots.txt data came from a fresh fetch or the cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheSource {
    Fresh,
    Cached,
}

/// The outcome of a compliance check for a single URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsDecision {
    pub url: String,
    pub allowed: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawl_delay_secs: Option<f64>,
    pub matched_agent: String,
    pub source: CacheSource,
}

/// Errors that can occur during robots.txt compliance checking.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum RobotsError {
    #[error("{message}")]
    Fetch {
        message: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    #[error("{message}")]
    Parse { message: String, url: String },

    #[error("{message}")]
    InvalidUrl { message: String, url: String },
}
