pub mod recipe_extraction;
pub mod recipe_tagging;
pub mod storage;
pub mod url_ingestion;

use recipe_extraction::extract_recipe;
use recipe_tagging::{extract_and_tag, tag_recipe};
use url_ingestion::{create_http_client, ingest_url, validate_url};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create HTTP client
    let http_client = create_http_client().expect("Failed to create HTTP client");

    tauri::Builder::default()
        .manage(http_client)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize SQLite database
            use tauri::Manager;
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to resolve app data directory");
            let db = storage::Database::new(&app_data_dir)
                .expect("Failed to initialize database");
            app.manage(db);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ingest_url,
            validate_url,
            extract_recipe,
            tag_recipe,
            extract_and_tag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
