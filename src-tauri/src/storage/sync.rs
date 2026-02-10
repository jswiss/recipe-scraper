use std::path::Path;

use super::change_log::{self, ChangeEntry};
use super::database::Database;
use super::models::*;

/// Exports pending change log entries to a JSONL file in the sync directory.
pub fn sync_export(db: &Database, sync_dir: &Path) -> Result<i64, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let pending = change_log::query_pending(&conn)?;
    if pending.is_empty() {
        return Ok(0);
    }

    std::fs::create_dir_all(sync_dir).map_err(|e| StorageError::Storage {
        message: format!("Failed to create sync directory: {e}"),
    })?;

    let file_path = sync_dir.join(format!("changes-{}.jsonl", db.device_id));
    let mut content = String::new();

    // Append mode: read existing content first
    if file_path.exists() {
        content = std::fs::read_to_string(&file_path).unwrap_or_default();
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
    }

    let max_id = pending.last().map(|e| e.id).unwrap_or(0);

    for entry in &pending {
        let line = serde_json::to_string(entry).map_err(|e| StorageError::Storage {
            message: format!("Failed to serialize change: {e}"),
        })?;
        content.push_str(&line);
        content.push('\n');
    }

    std::fs::write(&file_path, &content).map_err(|e| StorageError::Storage {
        message: format!("Failed to write sync file: {e}"),
    })?;

    change_log::mark_synced(&conn, max_id)?;

    Ok(pending.len() as i64)
}

/// Imports change log entries from remote device JSONL files.
pub fn sync_import(db: &Database, sync_dir: &Path) -> Result<i64, StorageError> {
    if !sync_dir.exists() {
        return Ok(0);
    }

    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let own_file = format!("changes-{}.jsonl", db.device_id);
    let mut total_imported = 0i64;

    let entries = std::fs::read_dir(sync_dir).map_err(|e| StorageError::Storage {
        message: format!("Failed to read sync directory: {e}"),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| StorageError::Storage {
            message: format!("Failed to read dir entry: {e}"),
        })?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip own device file
        if file_name == own_file {
            continue;
        }
        if !file_name.starts_with("changes-") || !file_name.ends_with(".jsonl") {
            continue;
        }

        let content = std::fs::read_to_string(entry.path()).map_err(|e| StorageError::Storage {
            message: format!("Failed to read {file_name}: {e}"),
        })?;

        // Get the device_id from filename
        let remote_device = file_name
            .strip_prefix("changes-")
            .and_then(|s| s.strip_suffix(".jsonl"))
            .unwrap_or("unknown")
            .to_string();

        // Get last imported ID for this device
        let last_id: i64 = conn
            .query_row(
                "SELECT last_imported_id FROM sync_state WHERE device_id = ?1",
                rusqlite::params![remote_device],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let mut max_imported_id = last_id;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: ChangeEntry = match serde_json::from_str(line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip already-imported entries
            if entry.id <= last_id {
                continue;
            }

            // Insert into local change log (with synced=1 since it came from remote)
            conn.execute(
                "INSERT INTO change_log (recipe_id, field_name, field_value, modified_at, device_id, synced)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1)",
                rusqlite::params![entry.recipe_id, entry.field_name, entry.field_value, entry.modified_at, entry.device_id],
            )
            .map_err(|e| StorageError::Storage {
                message: format!("Failed to import change: {e}"),
            })?;

            if entry.id > max_imported_id {
                max_imported_id = entry.id;
            }
            total_imported += 1;
        }

        // Update sync_state
        if max_imported_id > last_id {
            let now = change_log::now_utc();
            conn.execute(
                "INSERT OR REPLACE INTO sync_state (device_id, last_imported_id, last_import_at)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![remote_device, max_imported_id, now],
            )
            .map_err(|e| StorageError::Storage {
                message: format!("Failed to update sync state: {e}"),
            })?;
        }
    }

    Ok(total_imported)
}

/// Merges imported changes into the recipes table using per-field LWW.
pub fn merge_changes(db: &Database) -> Result<i64, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    // Find all recipe_ids with remote changes (device_id != our device)
    let mut stmt = conn
        .prepare("SELECT DISTINCT recipe_id FROM change_log WHERE device_id != ?1")
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    let recipe_ids: Vec<String> = stmt
        .query_map(rusqlite::params![db.device_id], |row| row.get(0))
        .map_err(|e| StorageError::Storage {
            message: format!("Query failed: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Collect failed: {e}"),
        })?;

    let mut merged = 0i64;

    for recipe_id in &recipe_ids {
        // For each field, get the latest change (highest modified_at, device_id tiebreaker)
        let mut field_stmt = conn
            .prepare(
                "SELECT field_name, field_value, modified_at, device_id FROM change_log
                 WHERE recipe_id = ?1
                 ORDER BY field_name, modified_at DESC, device_id DESC",
            )
            .map_err(|e| StorageError::Storage {
                message: format!("Failed to prepare: {e}"),
            })?;

        let changes: Vec<(String, Option<String>, String, String)> = field_stmt
            .query_map(rusqlite::params![recipe_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| StorageError::Storage {
                message: format!("Query failed: {e}"),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StorageError::Storage {
                message: format!("Collect failed: {e}"),
            })?;

        // Group by field_name, keep only the latest per field
        let mut latest_by_field: std::collections::HashMap<
            String,
            (Option<String>, String, String),
        > = std::collections::HashMap::new();
        for (field, value, ts, dev) in &changes {
            latest_by_field
                .entry(field.clone())
                .or_insert_with(|| (value.clone(), ts.clone(), dev.clone()));
        }

        // Check for delete-vs-modify: if __deleted exists but other fields are newer, restore
        let deleted_ts = latest_by_field
            .get("__deleted")
            .map(|(_, ts, _)| ts.clone());
        let has_newer_modify = if let Some(del_ts) = &deleted_ts {
            latest_by_field
                .iter()
                .any(|(k, (_, ts, _))| k != "__deleted" && ts > del_ts)
        } else {
            false
        };

        // Apply the latest value for each field
        let recipe_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM recipes WHERE id = ?1",
                rusqlite::params![recipe_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !recipe_exists {
            // Recipe doesn't exist locally — skip (it will be created when the full data arrives)
            continue;
        }

        // Handle deletion
        if let Some((_, _, _)) = latest_by_field.get("__deleted") {
            if !has_newer_modify {
                // Delete wins
                conn.execute(
                    "UPDATE recipes SET deleted = 1 WHERE id = ?1",
                    rusqlite::params![recipe_id],
                )
                .map_err(|e| StorageError::Storage {
                    message: format!("Merge failed: {e}"),
                })?;
                merged += 1;
                continue;
            } else {
                // Modify wins — restore
                conn.execute(
                    "UPDATE recipes SET deleted = 0 WHERE id = ?1",
                    rusqlite::params![recipe_id],
                )
                .map_err(|e| StorageError::Storage {
                    message: format!("Merge failed: {e}"),
                })?;
            }
        }

        // Apply field updates
        for (field, (value, _, _)) in &latest_by_field {
            if field == "__deleted" {
                continue;
            }
            match field.as_str() {
                "title" => {
                    conn.execute(
                        "UPDATE recipes SET title = ?1, title_status = 'found', updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![value, change_log::now_utc(), recipe_id],
                    ).ok();
                }
                "description" => {
                    conn.execute(
                        "UPDATE recipes SET description = ?1, description_status = 'found', updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![value, change_log::now_utc(), recipe_id],
                    ).ok();
                }
                "notes" => {
                    conn.execute(
                        "UPDATE recipes SET notes = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![
                            value.as_deref().unwrap_or(""),
                            change_log::now_utc(),
                            recipe_id
                        ],
                    )
                    .ok();
                }
                "servings" | "prep_time_minutes" | "cook_time_minutes" => {
                    conn.execute(
                        &format!("UPDATE recipes SET {field} = ?1, updated_at = ?2 WHERE id = ?3"),
                        rusqlite::params![value, change_log::now_utc(), recipe_id],
                    )
                    .ok();
                }
                _ => {} // Complex fields (ingredients, instructions, tags) handled separately
            }
        }

        merged += 1;
    }

    Ok(merged)
}

/// Full sync cycle: export + import + merge.
pub fn trigger_sync(db: &Database, sync_dir: &Path) -> Result<SyncResult, StorageError> {
    let exported = sync_export(db, sync_dir)?;
    let imported = sync_import(db, sync_dir)?;
    let merged = merge_changes(db)?;
    Ok(SyncResult {
        exported,
        imported,
        merged,
    })
}

pub fn get_sync_status(db: &Database) -> Result<SyncStatus, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let pending_changes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM change_log WHERE synced = 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn
        .prepare("SELECT device_id, last_import_at FROM sync_state")
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    let known_devices: Vec<KnownDevice> = stmt
        .query_map([], |row| {
            Ok(KnownDevice {
                device_id: row.get(0)?,
                last_imported_at: row.get(1)?,
            })
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Query failed: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Collect failed: {e}"),
        })?;

    let last_sync_at = known_devices
        .iter()
        .filter_map(|d| d.last_imported_at.as_ref())
        .max()
        .cloned();

    Ok(SyncStatus {
        enabled: true,
        pending_changes,
        last_sync_at,
        known_devices,
    })
}

#[cfg(test)]
mod tests {
    use super::super::repository;
    use super::*;
    use crate::recipe_extraction::{
        ExtractedField, ExtractedRecipe, ExtractionSource, Ingredient, Instruction,
    };
    use crate::recipe_tagging::{Tag, TagSet};

    fn test_db() -> Database {
        Database::new_in_memory().expect("Failed to create test DB")
    }

    fn save_sample(db: &Database) -> String {
        let recipe = ExtractedRecipe {
            title: ExtractedField::found("Test Pasta".into()),
            description: ExtractedField::found("Delicious".into()),
            ingredients: vec![Ingredient::new(
                "pasta",
                Some(200.0),
                Some("g".into()),
                "200g pasta",
            )],
            instructions: vec![Instruction::new(1, "Cook it")],
            prep_time_minutes: ExtractedField::found(10),
            cook_time_minutes: ExtractedField::found(20),
            servings: ExtractedField::found("4".into()),
            images: ExtractedField::not_found("none"),
            nutrition: ExtractedField::not_found("none"),
            source: ExtractionSource::JsonLd,
        };
        let tags = TagSet {
            cuisine: vec![Tag::new("Italian", 0.9)],
            course: vec![],
            diet: vec![],
        };
        repository::save_recipe(
            db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/test",
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn change_log_records_mutations() {
        let db = test_db();
        let _id = save_sample(&db);

        let conn = db.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM change_log", [], |r| r.get(0))
            .unwrap();
        assert!(count > 0, "change_log should have entries after save");
    }

    #[test]
    fn jsonl_export_import_round_trip() {
        let db = test_db();
        let _id = save_sample(&db);

        let dir = std::env::temp_dir().join("recipe_sync_test");
        std::fs::create_dir_all(&dir).unwrap();

        // Export
        let exported = sync_export(&db, &dir).unwrap();
        assert!(exported > 0);

        // Verify file exists
        let file = dir.join(format!("changes-{}.jsonl", db.device_id));
        assert!(file.exists());

        // Second export should have 0 (all marked synced)
        let exported2 = sync_export(&db, &dir).unwrap();
        assert_eq!(exported2, 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_vs_modify_restores_recipe() {
        let db = test_db();
        let id = save_sample(&db);

        // Delete the recipe
        repository::delete_recipe(&db, &id).unwrap();

        // Simulate a remote modify with a later timestamp
        {
            let conn = db.conn.lock().unwrap();
            // Insert a remote change that's "newer" than the delete
            conn.execute(
                "INSERT INTO change_log (recipe_id, field_name, field_value, modified_at, device_id, synced)
                 VALUES (?1, 'title', 'Updated Title', '9999-12-31T23:59:59.999999Z', 'remote-device', 1)",
                rusqlite::params![id],
            ).unwrap();
        }

        // Merge should restore the recipe
        let merged = merge_changes(&db).unwrap();
        assert!(merged > 0);

        // Recipe should be un-deleted
        let recipe = repository::get_recipe(&db, &id).unwrap();
        assert_eq!(recipe.title, ExtractedField::found("Updated Title".into()));
    }

    #[test]
    fn lww_merge_picks_newer_timestamp() {
        let db = test_db();
        let id = save_sample(&db);

        // Insert a remote change with a later timestamp
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO change_log (recipe_id, field_name, field_value, modified_at, device_id, synced)
                 VALUES (?1, 'notes', 'Remote notes', '9999-12-31T23:59:59.999999Z', 'remote-device', 1)",
                rusqlite::params![id],
            ).unwrap();
        }

        merge_changes(&db).unwrap();

        let recipe = repository::get_recipe(&db, &id).unwrap();
        assert_eq!(recipe.notes, "Remote notes");
    }
}
