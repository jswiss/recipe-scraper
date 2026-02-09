# Tasks: Persistent State

**Input**: Design documents from `/specs/005-persistent-state/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/tauri-commands.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependency and create module skeleton

- [ ] T001 Add `rusqlite = { version = "0.38", features = ["bundled"] }` to src-tauri/Cargo.toml and create src-tauri/src/storage/mod.rs with submodule declarations (models, database, repository, change_log, sync, export, backup, commands)
- [ ] T002 [P] Define StorageError (thiserror enum with Storage, NotFound variants), SavedRecipe, RecipeSummary, SaveResult, SearchQuery, ExportResult, ImportResult, SyncResult, and SyncStatus types in src-tauri/src/storage/models.rs — follow existing serde/thiserror patterns from url_ingestion/models.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Database infrastructure that MUST be complete before ANY user story

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Write SQL migration with all tables (recipes, ingredients, instructions, tags, change_log, sync_state, schema_version) and indexes per data-model.md in src-tauri/src/storage/migrations/001_initial.sql
- [ ] T004 Implement Database struct wrapping Mutex<Connection>, with new() that opens SQLite at app_data_dir/recipes.db, sets PRAGMA journal_mode=WAL + foreign_keys=ON + busy_timeout=5000, and runs migrations in src-tauri/src/storage/database.rs
- [ ] T005 Register `pub mod storage` in src-tauri/src/lib.rs, initialize Database inside .setup() closure using app.path().app_data_dir(), add to .manage(), and verify cargo build succeeds

**Checkpoint**: Database initializes on app startup, schema is created, cargo build passes

---

## Phase 3: User Story 1 - Save and Retrieve Recipes (Priority: P1) MVP

**Goal**: Recipes persist across app restarts with all fields preserved. Dedup by source URL. Support edit and soft delete.

**Independent Test**: Scrape a recipe, close the app, reopen it, verify all data intact.

### Implementation for User Story 1

- [ ] T006 [US1] Implement save_recipe (generate UUID, insert recipe row + ingredients + instructions + tags in a transaction; upsert if source_url exists per FR-003) and get_recipe (SELECT with JOINs for ingredients, instructions, tags; reconstruct ExtractedField enums) in src-tauri/src/storage/repository.rs
- [ ] T007 [US1] Implement update_recipe (partial field update: only update provided fields, handle notes per FR-016, replace ingredients/instructions/tags if provided, bump updated_at) and delete_recipe (SET deleted=1 soft delete per FR-009) in src-tauri/src/storage/repository.rs
- [ ] T008 [US1] Implement save_recipe, get_recipe, update_recipe, delete_recipe Tauri commands in src-tauri/src/storage/commands.rs following the existing State<> pattern, and register all four in generate_handler! in src-tauri/src/lib.rs
- [ ] T009 [US1] Add tests in src-tauri/src/storage/repository.rs: save→get round-trip preserves all fields, save same URL twice updates instead of duplicating (FR-003), update_recipe changes only specified fields, delete_recipe sets soft delete flag, get_recipe returns NotFound for deleted recipes

**Checkpoint**: User Story 1 fully functional — recipes persist, dedup works, edit and delete work

---

## Phase 4: User Story 2 - Browse and Search Offline (Priority: P2)

**Goal**: List all saved recipes and search/filter by title, ingredients, and tags. All operations < 100ms.

**Independent Test**: Save several recipes, call list_recipes and search_recipes with various queries, verify correct results return instantly.

### Implementation for User Story 2

- [ ] T010 [US2] Implement list_recipes (SELECT non-deleted recipes with tags, sorted by updated_at DESC) and search_recipes (case-insensitive LIKE on title + ingredient names, AND filtering on cuisine/course/diet tag labels) in src-tauri/src/storage/repository.rs
- [ ] T011 [US2] Implement list_recipes and search_recipes Tauri commands in src-tauri/src/storage/commands.rs and register in generate_handler! in src-tauri/src/lib.rs
- [ ] T012 [US2] Add tests in src-tauri/src/storage/repository.rs: list returns all non-deleted sorted by updated_at, search by title substring, search by ingredient name, filter by cuisine tag, filter by multiple tags (AND logic), combined text+tag query

**Checkpoint**: User Story 2 fully functional — browse and search work offline with correct filtering

---

## Phase 5: User Story 3 - Export for Other Apps (Priority: P3)

**Goal**: Export/import recipes as schema.org/Recipe JSON. Backup/restore entire collection.

**Independent Test**: Export a recipe, verify JSON matches schema.org/Recipe spec, import it on fresh DB, verify 100% fidelity.

### Implementation for User Story 3

- [ ] T013 [P] [US3] Create SchemaOrgRecipe, SchemaOrgHowToStep, SchemaOrgNutrition serialization structs with #[serde(rename_all = "camelCase")] and implement bidirectional conversion (SavedRecipe ↔ SchemaOrgRecipe) in src-tauri/src/storage/export.rs
- [ ] T014 [US3] Implement export_recipes (load from DB → convert → write JSON array to file) and import_recipes (read file → parse → dedup by source_url → save to DB, return imported/updated/skipped counts) in src-tauri/src/storage/export.rs
- [ ] T015 [P] [US3] Implement backup_collection (use rusqlite backup API to copy DB to file_path) and restore_collection (validate backup integrity, replace current DB) in src-tauri/src/storage/backup.rs
- [ ] T016 [US3] Implement export_recipes, import_recipes, backup_collection, restore_collection Tauri commands in src-tauri/src/storage/commands.rs and register in generate_handler! in src-tauri/src/lib.rs
- [ ] T017 [US3] Add tests in src-tauri/src/storage/export.rs: export→import round-trip with 100% data fidelity (SC-004), schema.org JSON structure validation (@context, @type fields), import dedup by source_url, import with malformed entries returns errors for skipped items

**Checkpoint**: User Story 3 fully functional — export/import preserves all data, backup/restore works

---

## Phase 6: User Story 4 - Sync Across Devices (Priority: P4)

**Goal**: Change log records all mutations. Export/import JSONL files via iCloud. Per-field LWW merge resolves conflicts.

**Independent Test**: Simulate two devices by creating change logs with overlapping edits, run merge, verify per-field LWW produces correct result and delete-vs-modify resolves per spec.

### Implementation for User Story 4

- [ ] T018 [US4] Implement append_change (record recipe_id, field_name, field_value, timestamp, device_id), query_pending (WHERE synced=0), and mark_synced (SET synced=1) in src-tauri/src/storage/change_log.rs
- [ ] T019 [US4] Integrate change_log recording into save_recipe, update_recipe, and delete_recipe — each write appends per-field change_log entries within the same transaction in src-tauri/src/storage/repository.rs
- [ ] T020 [US4] Implement sync_export (SELECT pending changes → write JSONL to iCloud container path as changes-{device_id}.jsonl → mark synced) and sync_import (read remote device JSONL files → INSERT into change_log) in src-tauri/src/storage/sync.rs
- [ ] T021 [US4] Implement merge_changes: for each imported change, compare modified_at with local field timestamp, apply if remote is newer (LWW); handle __deleted field where modify-wins-over-delete per spec clarification in src-tauri/src/storage/sync.rs
- [ ] T022 [US4] Implement trigger_sync (export + import + merge) and get_sync_status (pending count, last sync time, known devices) Tauri commands in src-tauri/src/storage/commands.rs and register in generate_handler! in src-tauri/src/lib.rs
- [ ] T023 [US4] Add tests in src-tauri/src/storage/sync.rs: change_log records all mutations, JSONL export/import round-trip, LWW merge picks newer timestamp, identical timestamps use device_id tiebreaker, delete-vs-modify conflict restores recipe (modify wins)

**Checkpoint**: User Story 4 fully functional — sync exports/imports changes, conflicts resolve correctly

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Performance validation and code quality

- [ ] T024 Validate <100ms performance for list_recipes and search_recipes with 5,000 synthetic recipes (insert in a loop, time queries) in src-tauri/src/storage/repository.rs tests (SC-002)
- [ ] T025 [P] Verify atomic transaction safety: test that a panic/error mid-save leaves DB unchanged (FR-008) in src-tauri/src/storage/repository.rs tests
- [ ] T026 Run cargo clippy and cargo fmt across all src-tauri/src/storage/ files, fix any warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — MVP milestone
- **US2 (Phase 4)**: Depends on US1 (needs saved recipes to list/search)
- **US3 (Phase 5)**: Depends on US1 (needs saved recipes to export)
- **US4 (Phase 6)**: Depends on US1 (needs write operations to record change log)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Blocked by Phase 2 only — no other story dependencies
- **US2 (P2)**: Depends on US1 (repository.rs read functions build on write functions)
- **US3 (P3)**: Depends on US1 (export reads from DB populated by save). Can run parallel with US2.
- **US4 (P4)**: Depends on US1 (change_log integrates into write operations). Can run parallel with US2/US3.

### Within Each User Story

- Repository functions before commands
- Commands before registration in lib.rs
- Tests after implementation

### Parallel Opportunities

- **Phase 1**: T001 || T002 (Cargo.toml/mod.rs vs models.rs)
- **Phase 5**: T013 || T015 (export.rs vs backup.rs — different files, no dependencies)
- **Phase 7**: T024 || T025 (different test concerns)
- **Cross-phase**: US2 and US3 can run in parallel after US1 completes. US4 can start after US1 completes.

---

## Parallel Example: User Story 3

```
# These two tasks can run simultaneously (different files):
T013: SchemaOrgRecipe structs + conversion in export.rs
T015: backup/restore using SQLite backup API in backup.rs

# Then sequentially:
T014: export/import functions in export.rs (needs T013)
T016: Tauri commands in commands.rs (needs T014 + T015)
T017: Tests in export.rs (needs T014)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T005)
3. Complete Phase 3: User Story 1 (T006–T009)
4. **STOP and VALIDATE**: Save a recipe, close app, reopen, verify data persists
5. Ship MVP — recipes now survive restarts

### Incremental Delivery

1. Setup + Foundational → Database ready
2. US1 → Recipes persist (MVP!)
3. US2 → Browse and search offline
4. US3 → Export/import + backup (can parallel with US2)
5. US4 → Sync across devices
6. Polish → Performance + safety verification

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story is independently testable after its checkpoint
- Commit after each task completion
- Total new dependency: 1 (rusqlite with bundled feature)
- All database operations use transactions for atomicity (FR-008)
