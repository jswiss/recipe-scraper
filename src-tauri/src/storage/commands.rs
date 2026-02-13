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
pub async fn delete_recipe(
    id: String,
    db: State<'_, Database>,
) -> Result<DeleteResult, StorageError> {
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

#[cfg(test)]
mod tests {
    use crate::recipe_extraction::{
        ExtractedField, ExtractedRecipe, ExtractionSource, Ingredient, Instruction,
    };
    use crate::recipe_tagging::{Tag, TagSet};
    use crate::storage::database::Database;
    use crate::storage::models::{SaveRecipeInput, StorageError};
    use crate::storage::repository;

    fn test_db() -> Database {
        Database::new_in_memory().expect("Failed to create test DB")
    }

    fn sample_recipe() -> ExtractedRecipe {
        ExtractedRecipe {
            title: ExtractedField::found("Test Recipe".to_string()),
            description: ExtractedField::found("A test recipe".to_string()),
            ingredients: vec![Ingredient::new(
                "flour",
                Some(2.0),
                Some("cups".into()),
                "2 cups flour",
            )],
            instructions: vec![Instruction::new(1, "Mix")],
            prep_time_minutes: ExtractedField::found(10),
            cook_time_minutes: ExtractedField::found(20),
            servings: ExtractedField::found("4 servings".to_string()),
            images: ExtractedField::not_found("Not provided"),
            nutrition: ExtractedField::not_found("Not provided"),
            source: ExtractionSource::JsonLd,
        }
    }

    fn sample_tags() -> TagSet {
        TagSet {
            cuisine: vec![Tag::new("Italian", 0.9)],
            course: vec![Tag::new("dinner", 0.8)],
            diet: vec![],
        }
    }

    #[test]
    fn test_save_recipe_command_returns_save_result() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();

        let result = repository::save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/test",
            },
        )
        .expect("save_recipe should succeed");

        assert!(!result.id.is_empty(), "ID should not be empty");
        assert!(result.created, "Should be a new recipe");
    }

    #[test]
    fn test_get_recipe_not_found_returns_error() {
        let db = test_db();
        let result = repository::get_recipe(&db, "nonexistent-id");

        assert!(result.is_err(), "Should return an error for missing ID");
        assert!(
            matches!(result.unwrap_err(), StorageError::NotFound { .. }),
            "Error should be NotFound"
        );
    }

    #[test]
    fn test_delete_recipe_returns_result() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();

        let save_result = repository::save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/delete-test",
            },
        )
        .expect("save_recipe should succeed");

        let delete_result =
            repository::delete_recipe(&db, &save_result.id).expect("delete_recipe should succeed");
        assert!(delete_result.deleted, "Recipe should be marked as deleted");

        let get_result = repository::get_recipe(&db, &save_result.id);
        assert!(
            matches!(get_result, Err(StorageError::NotFound { .. })),
            "Deleted recipe should return NotFound"
        );
    }
}
