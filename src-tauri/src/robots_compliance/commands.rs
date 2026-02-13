use tauri::State;

use crate::robots_compliance::checker::check_compliance;
use crate::robots_compliance::models::{RobotsDecision, RobotsError};
use crate::storage::Database;
use crate::url_ingestion::HttpClient;

/// Checks robots.txt compliance for a URL.
///
/// Returns a decision object indicating whether scraping is allowed or disallowed,
/// along with crawl delay and the matched user-agent group.
#[tauri::command]
pub async fn check_robots_compliance(
    url: String,
    client: State<'_, HttpClient>,
    db: State<'_, Database>,
) -> Result<RobotsDecision, RobotsError> {
    check_compliance(&client.0, &db, &url).await
}
