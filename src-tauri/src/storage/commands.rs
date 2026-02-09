use tauri::State;

use crate::recipe_extraction::ExtractedRecipe;
use crate::recipe_tagging::TagSet;

use super::database::Database;
use super::models::*;
use super::repository;

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
