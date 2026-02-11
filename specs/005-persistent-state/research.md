# Research: Persistent State

**Feature Branch**: `005-persistent-state`
**Date**: 2026-02-09

## Decision 1: SQLite Crate Selection

**Decision**: Use `rusqlite` with `bundled` feature.

**Rationale**: Thin Rust bindings over the SQLite C API. The `bundled` feature compiles SQLite from source, ensuring consistent version across platforms. Synchronous API is honest about what SQLite actually does — wrapping in `spawn_blocking` where needed is explicit. Full access to PRAGMA statements, backup API, and custom functions needed for WAL mode, export, and performance tuning.

**Alternatives considered**:

| Crate | Why rejected |
|-------|--------------|
| sqlx | Overkill. Adds async SQLite wrapping (just `spawn_blocking` under the hood), compile-time query checking requiring a live DB at build time, and multi-database abstraction with no practical gain. Heavy transitive dependencies. |
| diesel | Wrong tool. ORM adds complexity without proportional benefit for a simple 5-6 table schema. Query builder DSL obscures straightforward SQL. Requires CLI tool for migrations. Violates AHA and Minimal Dependencies principles. |
| tauri-plugin-sql | Uses sqlx internally (inherits its weight). Designed for frontend-driven DB access, but our operations are backend-driven (scrape → extract → tag → persist). Less control over SQLite configuration. |

**New dependency**: `rusqlite = { version = "0.38", features = ["bundled"] }`

---

## Decision 2: Sync Architecture

**Decision**: Change log pattern with iCloud file transport. The local SQLite database is NOT synced directly. Instead, an append-only change log is exported to iCloud as per-device files.

**Rationale**: Syncing raw SQLite files via iCloud is problematic because WAL mode creates 3 files (`.db`, `-wal`, `-shm`) that sync independently and can cause corruption. Even in DELETE journal mode, simultaneous writes on two devices create "conflicted copy" files requiring manual resolution. The change log approach keeps the local database fast (WAL mode, no cloud interference) and uses iCloud only as dumb file transport for small change files.

**Architecture**:
```
Local Device                          iCloud Container
-----------                          ----------------
recipes.db (WAL mode)                changes-{device_a}.jsonl
  ├── recipes table                  changes-{device_b}.jsonl
  ├── ingredients table
  ├── instructions table
  ├── tags table
  ├── change_log table ──export──>   (append new entries)
  └── sync_state table

On startup / file change:
  change_log <──import──             (read remote entries)
  recipes    <──merge───             (apply LWW per field)
```

**Alternatives considered**:

| Approach | Why rejected |
|----------|--------------|
| Direct SQLite file sync via iCloud | WAL mode creates 3 files that sync independently, causing corruption. DELETE mode has worse write performance and iCloud creates conflicted copies. |
| cr-sqlite (CRDT extension) | Adds a C extension dependency. Project is young (vlcn.io) with uncertain long-term maintenance. Loading custom extensions in Tauri requires care. Opaque complexity violates Readable & Simple principle. |
| CloudKit / NSPersistentCloudKitContainer | Requires Objective-C/Swift and Core Data. Not accessible from Rust without complex FFI. Ties to Apple ecosystem (violates open source and no vendor lock-in principles). |
| Custom HTTP sync server | Requires hosting infrastructure. Violates local-first principle (server becomes dependency). Overly complex for the recipe collection use case. |

---

## Decision 3: Conflict Resolution Strategy

**Decision**: Per-field last-write-wins via change log with UTC timestamps.

**Rationale**: Each mutation is recorded in a change log table with recipe ID, field name, new value, UTC timestamp (ISO 8601 with microsecond precision), and device ID. When merging remote changes, compare timestamps per field. Newest timestamp wins. For identical timestamps (extremely rare), device ID lexicographic comparison provides deterministic tiebreaker.

**Clock skew mitigation**: Modern Apple devices sync clocks via NTP to within ~10ms, which is more than sufficient for human-speed recipe editing.

**Delete vs modify**: A delete is recorded as a change log entry (`field = "__deleted"`, `value = "true"`). If a modify on another device has a later timestamp, the recipe is restored (per spec clarification).

**Alternatives considered**:

| Approach | Why rejected |
|----------|--------------|
| Per-row `modified_at` only | Cannot resolve field-level conflicts. Two devices editing different fields of the same recipe would cause one device's changes to be lost entirely. |
| Vector clocks | More complex to implement and debug with no practical benefit for a single-user-multi-device scenario where wall-clock timestamps are sufficiently accurate. |
| CRDTs (full) | Overkill for this use case. The data model is simple key-value fields, not collaborative editing of shared documents. LWW per field achieves the same result with dramatically less complexity. |

---

## Decision 4: Export Format

**Decision**: JSON using schema.org/Recipe vocabulary. Serialize using existing `serde` + `serde_json` (already in Cargo.toml).

**Rationale**: This is the same structured data format the app already extracts from websites (JSON-LD). The data model maps naturally. Widely supported by recipe apps. Human-readable. No new dependencies needed.

**Alternatives considered**:

| Format | Why rejected |
|--------|--------------|
| Custom JSON schema | No interoperability with other recipe apps. Users can't import into existing tools. |
| CSV | Loses structured data (ingredients, instructions are nested). Poor for round-trip fidelity. |
| Paprika format | Proprietary to one app. Limited ecosystem support. |
| json-ld crate | Overkill for generating output. Designed for consuming/transforming arbitrary JSON-LD, not serializing known structures. |

---

## Decision 5: Tauri Integration Pattern

**Decision**: Direct `rusqlite` via `Mutex<Connection>` in Tauri managed state, following the existing `HttpClient` pattern.

**Rationale**: The existing codebase wraps `reqwest::Client` in an `HttpClient` struct and manages it via `.manage()`. The database connection follows the exact same pattern: wrap `rusqlite::Connection` in a `Database` struct with `Mutex`, manage it as Tauri state, access via `State<'_, Database>` in commands.

**Migration strategy**: Simple version table with SQL strings executed on startup (~30 lines). No migration framework needed.

```rust
const MIGRATIONS: &[&str] = &[
    // v1: initial schema
    include_str!("../migrations/001_initial.sql"),
];
```

---

## Decision 6: Database Location

**Decision**: Store the primary database in Tauri's app data directory (`app.path().app_data_dir()`). Store change log export files in the iCloud container directory for sync.

**Rationale**: Tauri provides platform-appropriate data directories. The primary database stays local for performance. Only the lightweight change log files go to iCloud for sync.
