use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::recipe_extraction::{
    ExtractedField, ExtractedRecipe, ExtractionSource, Ingredient, Instruction, NutritionInfo,
};
use crate::recipe_tagging::{Tag, TagSet};

/// Errors from storage operations.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum StorageError {
    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("Recipe not found: {message}")]
    NotFound { message: String },
}

/// A fully persisted recipe with all related data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRecipe {
    pub id: String,
    pub source_url: String,
    pub title: ExtractedField<String>,
    pub description: ExtractedField<String>,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub prep_time_minutes: ExtractedField<u32>,
    pub cook_time_minutes: ExtractedField<u32>,
    pub servings: ExtractedField<String>,
    pub images: ExtractedField<Vec<String>>,
    pub nutrition: ExtractedField<NutritionInfo>,
    pub extraction_source: ExtractionSource,
    pub tags: TagSet,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Summary data for recipe list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    pub id: String,
    pub source_url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub prep_time_minutes: Option<u32>,
    pub cook_time_minutes: Option<u32>,
    pub tags: TagSet,
    pub created_at: String,
    pub updated_at: String,
}

/// Result of saving a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResult {
    pub id: String,
    pub created: bool,
}

/// Search query parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub cuisine_tags: Option<Vec<String>>,
    pub course_tags: Option<Vec<String>>,
    pub diet_tags: Option<Vec<String>>,
}

/// Fields that can be updated on a saved recipe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateFields {
    pub title: Option<String>,
    pub description: Option<String>,
    pub ingredients: Option<Vec<Ingredient>>,
    pub instructions: Option<Vec<Instruction>>,
    pub prep_time_minutes: Option<u32>,
    pub cook_time_minutes: Option<u32>,
    pub servings: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<TagSet>,
}

/// Result of an export operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub count: i64,
    pub file_path: String,
}

/// Result of an import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: i64,
    pub updated: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

/// Result of a backup operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub file_path: String,
    pub recipe_count: i64,
    pub size_bytes: i64,
}

/// Result of a restore operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub recipe_count: i64,
}

/// Result of a sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub exported: i64,
    pub imported: i64,
    pub merged: i64,
}

/// Current sync status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub enabled: bool,
    pub pending_changes: i64,
    pub last_sync_at: Option<String>,
    pub known_devices: Vec<KnownDevice>,
}

/// A known remote device for sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownDevice {
    pub device_id: String,
    pub last_imported_at: Option<String>,
}

/// Result of a delete operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub deleted: bool,
}

/// Result of an update operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub updated_at: String,
}

/// Input for saving a recipe from extraction + tagging pipeline.
pub struct SaveRecipeInput<'a> {
    pub recipe: &'a ExtractedRecipe,
    pub tags: &'a TagSet,
    pub source_url: &'a str,
}

/// Helper to convert ExtractedField to DB columns.
pub fn extracted_field_to_columns<T>(field: &ExtractedField<T>) -> (&str, Option<String>)
where
    T: std::fmt::Display,
{
    match field {
        ExtractedField::Found { value } => ("found", Some(value.to_string())),
        ExtractedField::NotFound { justification } => ("not_found", Some(justification.clone())),
    }
}

/// Helper to convert ExtractedField<u32> to DB columns.
pub fn extracted_u32_to_columns(field: &ExtractedField<u32>) -> (&str, Option<i64>) {
    match field {
        ExtractedField::Found { value } => ("found", Some(*value as i64)),
        ExtractedField::NotFound { .. } => ("not_found", None),
    }
}

/// Helper to get justification from an ExtractedField.
pub fn extracted_field_justification<T>(field: &ExtractedField<T>) -> Option<String> {
    match field {
        ExtractedField::Found { .. } => None,
        ExtractedField::NotFound { justification } => Some(justification.clone()),
    }
}

/// Helper to reconstruct an ExtractedField<String> from DB columns.
pub fn columns_to_extracted_string(
    status: &str,
    value: Option<String>,
    justification: Option<String>,
) -> ExtractedField<String> {
    match status {
        "found" => ExtractedField::Found {
            value: value.unwrap_or_default(),
        },
        _ => ExtractedField::NotFound {
            justification: justification.unwrap_or_else(|| "Not found in source".into()),
        },
    }
}

/// Helper to reconstruct an ExtractedField<u32> from DB columns.
pub fn columns_to_extracted_u32(
    status: &str,
    value: Option<i64>,
    justification: Option<String>,
) -> ExtractedField<u32> {
    match status {
        "found" => ExtractedField::Found {
            value: value.unwrap_or(0) as u32,
        },
        _ => ExtractedField::NotFound {
            justification: justification.unwrap_or_else(|| "Not found in source".into()),
        },
    }
}

/// Helper to reconstruct an ExtractedField<Vec<String>> from DB JSON column.
pub fn columns_to_extracted_vec_string(
    status: &str,
    json: Option<String>,
    justification: Option<String>,
) -> ExtractedField<Vec<String>> {
    match status {
        "found" => {
            let value = json
                .and_then(|j| serde_json::from_str(&j).ok())
                .unwrap_or_default();
            ExtractedField::Found { value }
        }
        _ => ExtractedField::NotFound {
            justification: justification.unwrap_or_else(|| "Not found in source".into()),
        },
    }
}

/// Helper to reconstruct an ExtractedField<NutritionInfo> from DB JSON column.
pub fn columns_to_extracted_nutrition(
    status: &str,
    json: Option<String>,
    justification: Option<String>,
) -> ExtractedField<NutritionInfo> {
    match status {
        "found" => {
            let value = json
                .and_then(|j| serde_json::from_str(&j).ok())
                .unwrap_or_default();
            ExtractedField::Found { value }
        }
        _ => ExtractedField::NotFound {
            justification: justification.unwrap_or_else(|| "Not found in source".into()),
        },
    }
}

/// Converts a TagSet into flat (domain, tag) pairs for DB storage.
pub fn tag_set_to_rows(tags: &TagSet) -> Vec<(&str, &Tag)> {
    let mut rows = Vec::new();
    for tag in &tags.cuisine {
        rows.push(("cuisine", tag));
    }
    for tag in &tags.course {
        rows.push(("course", tag));
    }
    for tag in &tags.diet {
        rows.push(("diet", tag));
    }
    rows
}

/// Reconstructs a TagSet from flat (domain, label, confidence) rows.
pub fn rows_to_tag_set(rows: &[(String, String, f64)]) -> TagSet {
    let mut tags = TagSet::empty();
    for (domain, label, confidence) in rows {
        let tag = Tag::new(label.clone(), *confidence);
        match domain.as_str() {
            "cuisine" => tags.cuisine.push(tag),
            "course" => tags.course.push(tag),
            "diet" => tags.diet.push(tag),
            _ => {}
        }
    }
    tags
}
