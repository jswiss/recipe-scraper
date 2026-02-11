# Quickstart: Persistent State

**Feature Branch**: `005-persistent-state`
**Date**: 2026-02-09

## Prerequisites

- Rust 1.77+ (stable toolchain)
- Existing recipe-scraper codebase on `005-persistent-state` branch

## New Dependency

Add to `src-tauri/Cargo.toml`:

```toml
# Local storage
rusqlite = { version = "0.38", features = ["bundled"] }
```

No other new dependencies are needed. The feature uses existing `serde`, `serde_json`, and `thiserror`.

## Module Structure

New module `storage` added alongside existing modules:

```
src-tauri/src/
├── main.rs
├── lib.rs                    # Add: mod storage, manage Database state
├── url_ingestion/            # Existing (unchanged)
├── recipe_extraction/        # Existing (unchanged)
├── recipe_tagging/           # Existing (unchanged)
└── storage/                  # NEW
    ├── mod.rs                # Module exports
    ├── models.rs             # SavedRecipe, StorageError, SyncStatus
    ├── database.rs           # Database struct, connection management, migrations
    ├── repository.rs         # CRUD operations (save, get, list, search, update, delete)
    ├── change_log.rs         # Change log recording and querying
    ├── sync.rs               # Export/import change logs, merge logic
    ├── export.rs             # Schema.org/Recipe JSON export/import
    ├── backup.rs             # SQLite backup/restore
    └── commands.rs           # Tauri command handlers
```

## Key Integration Points

### 1. Database initialization (lib.rs)

The `Database` state is created in `lib.rs::run()` alongside the existing `HttpClient`:

```rust
let db = Database::new(&app_data_dir).expect("Failed to initialize database");

tauri::Builder::default()
    .manage(http_client)
    .manage(db)  // NEW
    // ...
```

### 2. Command registration (lib.rs)

New commands added to `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // Existing
    ingest_url, validate_url, extract_recipe, tag_recipe, extract_and_tag,
    // NEW - Storage
    save_recipe, get_recipe, list_recipes, search_recipes,
    update_recipe, delete_recipe,
    // NEW - Export/Import
    export_recipes, import_recipes, backup_collection, restore_collection,
    // NEW - Sync
    trigger_sync, get_sync_status,
])
```

### 3. Database access pattern

Commands access the database via Tauri state, matching the existing `HttpClient` pattern:

```rust
#[tauri::command]
pub async fn save_recipe(
    recipe: ExtractedRecipe,
    tags: TagSet,
    source_url: String,
    db: State<'_, Database>,
) -> Result<SaveResult, StorageError> {
    // Lock, execute, return
}
```

## Build & Test

```bash
cd src-tauri
cargo build            # Verify compilation with rusqlite
cargo test             # Run all tests including new storage tests
cargo clippy           # Lint check
```

## Data Location

- **Primary database**: `{app_data_dir}/recipes.db` (Tauri app data directory)
- **Sync files**: `{icloud_container}/changes-{device_id}.jsonl` (when sync enabled)
- **Backups**: User-chosen file path via save dialog
