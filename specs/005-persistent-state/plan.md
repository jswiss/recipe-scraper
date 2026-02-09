# Implementation Plan: Persistent State

**Branch**: `005-persistent-state` | **Date**: 2026-02-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-persistent-state/spec.md`

## Summary

Add persistent local storage for recipes using SQLite via `rusqlite`, enabling recipes to survive app restarts, support offline browsing/search/filter, export/import as schema.org/Recipe JSON, and sync across devices via iCloud change log files with per-field last-write-wins conflict resolution.

## Technical Context

**Language/Version**: Rust 1.77+ (stable toolchain)
**Primary Dependencies**: tauri 2.1.0, serde 1.0, thiserror 2, rusqlite 0.38 (NEW — `bundled` feature)
**Storage**: SQLite (local file, WAL mode) via rusqlite
**Testing**: `cargo test` (unit + integration tests inline)
**Target Platform**: macOS desktop (Tauri 2.x)
**Project Type**: Single (Tauri desktop app with Rust backend)
**Performance Goals**: All local operations < 100ms with 5,000 recipes (SC-002)
**Constraints**: Fully offline-capable, no network required for local operations, atomic writes for crash safety
**Scale/Scope**: Up to 5,000 recipes per collection, single user per device, multi-device sync via iCloud

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | PASS | New `storage` module follows existing module pattern. Functions are single-purpose. SQL is explicit, not hidden behind ORM abstractions. |
| II. AHA Programming | PASS | No premature abstractions. Repository functions are direct SQL operations. No generic data access layer — purpose-built for recipe data. |
| III. Minimal Dependencies | PASS | One new dependency: `rusqlite` (bundled). Evaluated alternatives (sqlx, diesel, tauri-plugin-sql) and rejected for excess weight. See research.md. |
| IV. Accessibility First | N/A | Backend-only feature. No UI components in this scope. |
| V. Monorepo + Open Source | PASS | New module lives in existing `src-tauri/src/` alongside other modules. No new repositories or packages. iCloud sync avoids cloud vendor lock-in for sync protocol (file transport only). |
| VI. Local First | PASS | SQLite on device is primary. All operations work offline. iCloud sync is secondary and optional. No loading spinners for local ops. User retains full data ownership via export/backup. |

**Post-Phase 1 re-check**: All gates still pass. The change log sync pattern keeps the local database as primary truth. Export uses open standard (schema.org/Recipe JSON). No cloud services required for core functionality.

## Project Structure

### Documentation (this feature)

```text
specs/005-persistent-state/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: technology decisions
├── data-model.md        # Phase 1: SQLite schema
├── quickstart.md        # Phase 1: setup instructions
├── contracts/           # Phase 1: Tauri command contracts
│   └── tauri-commands.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/src/
├── main.rs                    # Entry point (unchanged)
├── lib.rs                     # Add: mod storage, Database state, new commands
├── url_ingestion/             # Existing (unchanged)
├── recipe_extraction/         # Existing (unchanged)
├── recipe_tagging/            # Existing (unchanged)
└── storage/                   # NEW MODULE
    ├── mod.rs                 # Module exports
    ├── models.rs              # SavedRecipe, StorageError, SaveResult, SyncStatus
    ├── database.rs            # Database struct, Mutex<Connection>, migrations
    ├── repository.rs          # CRUD: save, get, list, search, update, delete
    ├── change_log.rs          # Append entries, query pending, mark synced
    ├── sync.rs                # Export to iCloud, import from remote, merge LWW
    ├── export.rs              # Schema.org/Recipe JSON serialization, import parsing
    ├── backup.rs              # SQLite backup API wrappers
    └── commands.rs            # Tauri command handlers (thin wrappers)
```

**Structure Decision**: Follows the existing single-module-per-feature pattern (`url_ingestion/`, `recipe_extraction/`, `recipe_tagging/`). The new `storage/` module is added at the same level. Each file has a single responsibility. Commands are thin wrappers that delegate to business logic functions, matching the pattern in `recipe_tagging/commands.rs`.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
