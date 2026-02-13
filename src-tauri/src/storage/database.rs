use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use super::models::StorageError;

const MIGRATIONS: &[&str] = &[
    include_str!("migrations/001_initial.sql"),
    include_str!("migrations/002_robots_cache.sql"),
];

pub struct Database {
    pub(crate) conn: Mutex<Connection>,
    pub(crate) device_id: String,
}

impl Database {
    /// Opens (or creates) the SQLite database at `dir/recipes.db`, configures
    /// WAL mode and foreign keys, and runs any pending migrations.
    pub fn new(dir: &Path) -> Result<Self, StorageError> {
        std::fs::create_dir_all(dir).map_err(|e| StorageError::Storage {
            message: format!("Failed to create data directory: {e}"),
        })?;

        let db_path = dir.join("recipes.db");
        let conn = Connection::open(&db_path).map_err(|e| StorageError::Storage {
            message: format!("Failed to open database: {e}"),
        })?;

        // Configure SQLite pragmas
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to set pragmas: {e}"),
        })?;

        let db = Self {
            conn: Mutex::new(conn),
            device_id: generate_device_id(&db_path),
        };

        db.run_migrations()?;

        Ok(db)
    }

    /// Opens an in-memory database for testing.
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory().map_err(|e| StorageError::Storage {
            message: format!("Failed to open in-memory database: {e}"),
        })?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to set pragmas: {e}"),
        })?;

        let db = Self {
            conn: Mutex::new(conn),
            device_id: "test-device".to_string(),
        };

        db.run_migrations()?;

        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Storage {
            message: format!("Failed to acquire lock: {e}"),
        })?;

        // Ensure schema_version table exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to create schema_version table: {e}"),
        })?;

        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| StorageError::Storage {
                message: format!("Failed to read schema version: {e}"),
            })?;

        for (i, migration) in MIGRATIONS.iter().enumerate() {
            let version = (i + 1) as i64;
            if version > current_version {
                let tx = conn
                    .unchecked_transaction()
                    .map_err(|e| StorageError::Storage {
                        message: format!("Failed to begin migration {version} transaction: {e}"),
                    })?;

                tx.execute_batch(migration)
                    .map_err(|e| StorageError::Storage {
                        message: format!("Migration {version} failed: {e}"),
                    })?;

                let now = super::change_log::now_utc();
                tx.execute(
                    "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
                    rusqlite::params![version, now],
                )
                .map_err(|e| StorageError::Storage {
                    message: format!("Failed to record migration {version}: {e}"),
                })?;

                tx.commit().map_err(|e| StorageError::Storage {
                    message: format!("Failed to commit migration {version}: {e}"),
                })?;
            }
        }

        Ok(())
    }
}

/// Generates a stable device ID by hashing the database file path.
/// Falls back to a random UUID if the path can't be hashed.
fn generate_device_id(db_path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    db_path.hash(&mut hasher);
    let hash = hasher.finish();
    format!("device-{hash:016x}")
}
