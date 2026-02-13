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

#[cfg(test)]
mod tests {
    use crate::robots_compliance::checker::check_compliance;
    use crate::robots_compliance::models::RobotsError;
    use crate::storage::database::Database;

    #[tokio::test]
    async fn test_check_robots_invalid_url_returns_error() {
        let client = reqwest::Client::new();
        let db = Database::new_in_memory().unwrap();
        let result = check_compliance(&client, &db, "not-a-url").await;
        assert!(result.is_err(), "Invalid URL should produce an error");
        assert!(
            matches!(result.unwrap_err(), RobotsError::InvalidUrl { .. }),
            "Error should be InvalidUrl variant"
        );
    }
}
