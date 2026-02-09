use rusqlite::Connection;

use super::models::StorageError;

/// Appends a change log entry within an existing transaction.
/// The caller is responsible for transaction management.
pub fn append_change(
    conn: &Connection,
    recipe_id: &str,
    field_name: &str,
    field_value: Option<&str>,
    device_id: &str,
) -> Result<(), StorageError> {
    let now = chrono_now();
    conn.execute(
        "INSERT INTO change_log (recipe_id, field_name, field_value, modified_at, device_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![recipe_id, field_name, field_value, now, device_id],
    )
    .map_err(|e| StorageError::Storage {
        message: format!("Failed to append change log: {e}"),
    })?;
    Ok(())
}

/// A single change log entry for sync export/import.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangeEntry {
    pub id: i64,
    pub recipe_id: String,
    pub field_name: String,
    pub field_value: Option<String>,
    pub modified_at: String,
    pub device_id: String,
}

/// Returns all pending (unsynced) change log entries.
pub fn query_pending(conn: &Connection) -> Result<Vec<ChangeEntry>, StorageError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, recipe_id, field_name, field_value, modified_at, device_id
             FROM change_log WHERE synced = 0 ORDER BY id ASC",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare pending query: {e}"),
        })?;

    let entries = stmt
        .query_map([], |row| {
            Ok(ChangeEntry {
                id: row.get(0)?,
                recipe_id: row.get(1)?,
                field_name: row.get(2)?,
                field_value: row.get(3)?,
                modified_at: row.get(4)?,
                device_id: row.get(5)?,
            })
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to query pending changes: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to collect pending changes: {e}"),
        })?;

    Ok(entries)
}

/// Marks change log entries as synced (exported to iCloud).
pub fn mark_synced(conn: &Connection, up_to_id: i64) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE change_log SET synced = 1 WHERE id <= ?1 AND synced = 0",
        rusqlite::params![up_to_id],
    )
    .map_err(|e| StorageError::Storage {
        message: format!("Failed to mark changes as synced: {e}"),
    })?;
    Ok(())
}

/// Returns ISO 8601 UTC timestamp with microsecond precision.
fn chrono_now() -> String {
    // Use SystemTime for UTC timestamp without adding chrono dependency
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let micros = duration.subsec_micros();

    // Convert to date/time components
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Calculate date from days since epoch (1970-01-01)
    let (year, month, day) = days_to_date(days as i64);

    format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}.{micros:06}Z"
    )
}

/// Exposed for use by repository module.
pub fn now_utc() -> String {
    chrono_now()
}

fn days_to_date(days: i64) -> (i64, u32, u32) {
    // Civil calendar algorithm from Howard Hinnant
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
