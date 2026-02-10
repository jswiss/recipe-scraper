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
