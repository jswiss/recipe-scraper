use tauri::State;

use crate::recipe_extraction::ExtractedRecipe;
use crate::recipe_tagging::TagSet;

use super::backup;
use super::database::Database;
use super::export;
use super::models::*;
use super::repository;
use super::sync;

#[tauri::command]
pub async fn save_recipe(
    recipe: ExtractedRecipe,
    tags: TagSet,
    source_url: String,
    db: State<'_, Database>,
) -> Result<SaveResult, StorageError> {
    let input = SaveRecipeInput {
        recipe: &recipe,
        tags: &tags,
        source_url: &source_url,
    };
    repository::save_recipe(&db, input)
}

#[tauri::command]
pub async fn get_recipe(id: String, db: State<'_, Database>) -> Result<SavedRecipe, StorageError> {
    repository::get_recipe(&db, &id)
}

#[tauri::command]
pub async fn update_recipe(
    id: String,
    fields: UpdateFields,
    db: State<'_, Database>,
) -> Result<UpdateResult, StorageError> {
    repository::update_recipe(&db, &id, &fields)
}

#[tauri::command]
pub async fn delete_recipe(id: String, db: State<'_, Database>) -> Result<DeleteResult, StorageError> {
    repository::delete_recipe(&db, &id)
}

#[tauri::command]
pub async fn list_recipes(db: State<'_, Database>) -> Result<Vec<RecipeSummary>, StorageError> {
    repository::list_recipes(&db)
}

#[tauri::command]
pub async fn search_recipes(
    query: Option<String>,
    cuisine_tags: Option<Vec<String>>,
    course_tags: Option<Vec<String>>,
    diet_tags: Option<Vec<String>>,
    db: State<'_, Database>,
) -> Result<Vec<RecipeSummary>, StorageError> {
    let sq = SearchQuery {
        query,
        cuisine_tags,
        course_tags,
        diet_tags,
    };
    repository::search_recipes(&db, &sq)
}

#[tauri::command]
pub async fn export_recipes(
    recipe_ids: Option<Vec<String>>,
    file_path: String,
    db: State<'_, Database>,
) -> Result<ExportResult, StorageError> {
    export::export_recipes_to_file(&db, recipe_ids.as_deref(), &file_path)
}

#[tauri::command]
pub async fn import_recipes(
    file_path: String,
    db: State<'_, Database>,
) -> Result<ImportResult, StorageError> {
    export::import_recipes_from_file(&db, &file_path)
}

#[tauri::command]
pub async fn backup_collection(
    file_path: String,
    db: State<'_, Database>,
) -> Result<BackupResult, StorageError> {
    let (recipe_count, path) = backup::backup_collection_to(&db, &file_path)?;
    let size_bytes = std::fs::metadata(&path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    Ok(BackupResult {
        file_path: path,
        recipe_count,
        size_bytes,
    })
}

#[tauri::command]
pub async fn restore_collection(
    file_path: String,
    db: State<'_, Database>,
) -> Result<RestoreResult, StorageError> {
    let recipe_count = backup::restore_collection_from(&db, &file_path)?;
    Ok(RestoreResult { recipe_count })
}

#[tauri::command]
pub async fn trigger_sync(
    sync_dir: String,
    db: State<'_, Database>,
) -> Result<SyncResult, StorageError> {
    sync::trigger_sync(&db, std::path::Path::new(&sync_dir))
}

#[tauri::command]
pub async fn get_sync_status(db: State<'_, Database>) -> Result<SyncStatus, StorageError> {
    sync::get_sync_status(&db)
}
