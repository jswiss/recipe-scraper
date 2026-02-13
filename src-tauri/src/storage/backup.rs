use super::database::Database;
use super::models::StorageError;

pub fn backup_collection_to(db: &Database, file_path: &str) -> Result<(i64, String), StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let mut backup_conn =
        rusqlite::Connection::open(file_path).map_err(|e| StorageError::Storage {
            message: format!("Failed to open backup file: {e}"),
        })?;

    let backup = rusqlite::backup::Backup::new(&conn, &mut backup_conn).map_err(|e| {
        StorageError::Storage {
            message: format!("Failed to create backup: {e}"),
        }
    })?;

    backup
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|e| StorageError::Storage {
            message: format!("Backup failed: {e}"),
        })?;

    drop(backup);

    let recipe_count: i64 = backup_conn
        .query_row(
            "SELECT COUNT(*) FROM recipes WHERE deleted = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to count recipes in backup: {e}"),
        })?;

    Ok((recipe_count, file_path.to_string()))
}

pub fn restore_collection_from(db: &Database, file_path: &str) -> Result<i64, StorageError> {
    // Validate the backup by opening it and checking schema
    let backup_conn = rusqlite::Connection::open(file_path).map_err(|e| StorageError::Storage {
        message: format!("Failed to open backup file: {e}"),
    })?;

    let has_recipes: bool = backup_conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='recipes'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Invalid backup file: {e}"),
        })?;

    if !has_recipes {
        return Err(StorageError::Storage {
            message: "Invalid backup: no recipes table found".into(),
        });
    }

    let recipe_count: i64 = backup_conn
        .query_row(
            "SELECT COUNT(*) FROM recipes WHERE deleted = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to count recipes in backup: {e}"),
        })?;

    drop(backup_conn);

    // Restore: copy backup into current DB
    let mut conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let src = rusqlite::Connection::open(file_path).map_err(|e| StorageError::Storage {
        message: format!("Failed to open backup for restore: {e}"),
    })?;

    let backup =
        rusqlite::backup::Backup::new(&src, &mut conn).map_err(|e| StorageError::Storage {
            message: format!("Failed to create restore operation: {e}"),
        })?;

    backup
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|e| StorageError::Storage {
            message: format!("Restore failed: {e}"),
        })?;

    Ok(recipe_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::{
        ExtractedField, ExtractedRecipe, ExtractionSource, Ingredient, Instruction,
    };
    use crate::recipe_tagging::{Tag, TagSet};
    use crate::storage::database::Database;
    use crate::storage::models::SaveRecipeInput;
    use crate::storage::repository;

    fn temp_backup_path(suffix: &str) -> String {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!(
            "{}/recipe_backup_test_{}_{}.db",
            std::env::temp_dir().display(),
            suffix,
            ts
        )
    }

    fn sample_recipe(title: &str) -> ExtractedRecipe {
        ExtractedRecipe {
            title: ExtractedField::found(title.to_string()),
            description: ExtractedField::found("Test description".to_string()),
            ingredients: vec![Ingredient::new(
                "flour",
                Some(2.0),
                Some("cups".into()),
                "2 cups flour",
            )],
            instructions: vec![Instruction::new(1, "Mix ingredients")],
            prep_time_minutes: ExtractedField::found(10),
            cook_time_minutes: ExtractedField::found(20),
            servings: ExtractedField::found("4 servings".to_string()),
            images: ExtractedField::found(vec!["https://example.com/img.jpg".to_string()]),
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

    /// T014: backup roundtrip — save 3 recipes, backup, verify backup file contents.
    #[test]
    fn test_backup_roundtrip() {
        let db = Database::new_in_memory().unwrap();
        let tags = sample_tags();

        // Save 3 test recipes
        for i in 1..=3 {
            let recipe = sample_recipe(&format!("Recipe {i}"));
            repository::save_recipe(
                &db,
                SaveRecipeInput {
                    recipe: &recipe,
                    tags: &tags,
                    source_url: &format!("https://example.com/recipe/{i}"),
                },
            )
            .unwrap();
        }

        let path = temp_backup_path("roundtrip");

        let (count, returned_path) = backup_collection_to(&db, &path).unwrap();
        assert_eq!(count, 3);
        assert_eq!(returned_path, path);

        // Open the backup file directly and verify contents
        let backup_conn = rusqlite::Connection::open(&path).unwrap();
        let row_count: i64 = backup_conn
            .query_row(
                "SELECT COUNT(*) FROM recipes WHERE deleted = 0",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(row_count, 3);

        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    /// T015: restore from backup — backup populated DB, restore into fresh DB, verify recipes.
    #[test]
    fn test_restore_from_backup() {
        let source_db = Database::new_in_memory().unwrap();
        let tags = sample_tags();

        let mut saved_ids = Vec::new();
        for i in 1..=3 {
            let recipe = sample_recipe(&format!("Restore Recipe {i}"));
            let result = repository::save_recipe(
                &source_db,
                SaveRecipeInput {
                    recipe: &recipe,
                    tags: &tags,
                    source_url: &format!("https://example.com/restore/{i}"),
                },
            )
            .unwrap();
            saved_ids.push(result.id);
        }

        let path = temp_backup_path("restore");
        backup_collection_to(&source_db, &path).unwrap();

        // Create a fresh empty DB and restore into it
        let fresh_db = Database::new_in_memory().unwrap();
        let restored_count = restore_collection_from(&fresh_db, &path).unwrap();
        assert_eq!(restored_count, 3);

        // Verify each recipe can be retrieved and has correct fields
        for (i, id) in saved_ids.iter().enumerate() {
            let saved = repository::get_recipe(&fresh_db, id).unwrap();
            let expected_title = format!("Restore Recipe {}", i + 1);
            assert_eq!(saved.title, ExtractedField::found(expected_title));
            assert_eq!(saved.ingredients.len(), 1);
            assert_eq!(saved.ingredients[0].name, "flour");
            assert_eq!(saved.instructions.len(), 1);
            assert_eq!(saved.instructions[0].text, "Mix ingredients");
        }

        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    /// T016: restore corrupted backup returns StorageError::Storage.
    #[test]
    fn test_restore_corrupted_backup_returns_error() {
        let db = Database::new_in_memory().unwrap();
        let path = temp_backup_path("corrupted");

        // Write invalid data to the file
        std::fs::write(&path, "not a database").unwrap();

        let result = restore_collection_from(&db, &path);
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::Storage { message } => {
                assert!(!message.is_empty());
            }
            other => panic!("Expected StorageError::Storage, got: {other:?}"),
        }

        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    /// T017: restore empty file returns error, original data unaffected.
    #[test]
    fn test_restore_empty_file_returns_error() {
        let db = Database::new_in_memory().unwrap();
        let tags = sample_tags();

        // Save one recipe first
        let recipe = sample_recipe("Original Recipe");
        let result = repository::save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/original",
            },
        )
        .unwrap();
        let original_id = result.id;

        // Create a 0-byte temp file
        let path = temp_backup_path("empty");
        std::fs::write(&path, b"").unwrap();

        let restore_result = restore_collection_from(&db, &path);
        assert!(restore_result.is_err());

        // Verify the original recipe is still accessible
        let saved = repository::get_recipe(&db, &original_id).unwrap();
        assert_eq!(
            saved.title,
            ExtractedField::found("Original Recipe".to_string())
        );

        // Clean up
        let _ = std::fs::remove_file(&path);
    }
}
