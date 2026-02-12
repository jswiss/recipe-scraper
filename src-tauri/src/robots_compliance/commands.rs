use tauri::State;

use crate::robots_compliance::models::{RobotsDecision, RobotsError};
use crate::storage::Database;
use crate::url_ingestion::HttpClient;

/// Checks robots.txt compliance for a URL.
#[tauri::command]
pub async fn check_robots_compliance(
    _url: String,
    _client: State<'_, HttpClient>,
    _db: State<'_, Database>,
) -> Result<RobotsDecision, RobotsError> {
    // Placeholder — implemented in T008
    todo!()
}
