# Data Model: Persistent State

**Feature Branch**: `005-persistent-state`
**Date**: 2026-02-09

## Entity Relationship Overview

```
┌─────────────┐       ┌──────────────────┐
│   recipes    │──1:N──│   ingredients    │
│              │       └──────────────────┘
│              │       ┌──────────────────┐
│              │──1:N──│  instructions    │
│              │       └──────────────────┘
│              │       ┌──────────────────┐
│              │──1:N──│      tags        │
└─────────────┘       └──────────────────┘
       │
       │ mutations recorded in
       ▼
┌─────────────────┐
│   change_log    │
└─────────────────┘
       │
       │ sync metadata in
       ▼
┌─────────────────┐
│   sync_state    │
└─────────────────┘
```

## Tables

### recipes

The central entity. One row per scraped recipe.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | UUID v4, generated on first save |
| source_url | TEXT | NOT NULL, UNIQUE | Normalized URL used for deduplication (FR-003) |
| title | TEXT | | Recipe title (nullable if extraction returned NotFound) |
| title_status | TEXT | NOT NULL | "found" or "not_found" |
| title_justification | TEXT | | Justification when status is "not_found" |
| description | TEXT | | Recipe description |
| description_status | TEXT | NOT NULL | "found" or "not_found" |
| description_justification | TEXT | | |
| servings | TEXT | | Serving size/yield text |
| servings_status | TEXT | NOT NULL | "found" or "not_found" |
| servings_justification | TEXT | | |
| prep_time_minutes | INTEGER | | Preparation time in minutes |
| prep_time_status | TEXT | NOT NULL | "found" or "not_found" |
| prep_time_justification | TEXT | | |
| cook_time_minutes | INTEGER | | Cooking time in minutes |
| cook_time_status | TEXT | NOT NULL | "found" or "not_found" |
| cook_time_justification | TEXT | | |
| images_json | TEXT | | JSON array of image URLs |
| images_status | TEXT | NOT NULL | "found" or "not_found" |
| images_justification | TEXT | | |
| nutrition_json | TEXT | | JSON object of NutritionInfo |
| nutrition_status | TEXT | NOT NULL | "found" or "not_found" |
| nutrition_justification | TEXT | | |
| extraction_source | TEXT | NOT NULL | "json_ld", "microdata", or "ai_fallback" |
| notes | TEXT | NOT NULL DEFAULT '' | User's personal notes (FR-016) |
| created_at | TEXT | NOT NULL | ISO 8601 UTC timestamp (FR-010) |
| updated_at | TEXT | NOT NULL | ISO 8601 UTC timestamp (FR-010) |
| deleted | INTEGER | NOT NULL DEFAULT 0 | Soft delete flag for sync |
| device_id | TEXT | NOT NULL | Device that last modified this row |

**Uniqueness**: `source_url` is UNIQUE — re-scraping the same URL updates the existing row.

**Identity**: `id` (UUID) is the stable identity across devices for sync.

### ingredients

One row per ingredient per recipe. Ordered by `position`.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | Local row ID |
| recipe_id | TEXT | NOT NULL, FK → recipes(id) ON DELETE CASCADE | Parent recipe |
| position | INTEGER | NOT NULL | 0-indexed order |
| name | TEXT | NOT NULL | Ingredient name |
| quantity | REAL | | Parsed quantity (nullable) |
| unit | TEXT | | Parsed unit (nullable) |
| raw_text | TEXT | NOT NULL | Original raw text from source |

**Index**: `(recipe_id, position)` for ordered retrieval.

### instructions

One row per instruction step per recipe. Ordered by `step_number`.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | Local row ID |
| recipe_id | TEXT | NOT NULL, FK → recipes(id) ON DELETE CASCADE | Parent recipe |
| step_number | INTEGER | NOT NULL | 1-indexed step number |
| text | TEXT | NOT NULL | Instruction text |

**Index**: `(recipe_id, step_number)` for ordered retrieval.

### tags

One row per tag per recipe. Stores cuisine, course, and diet tags together.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | Local row ID |
| recipe_id | TEXT | NOT NULL, FK → recipes(id) ON DELETE CASCADE | Parent recipe |
| domain | TEXT | NOT NULL | "cuisine", "course", or "diet" |
| label | TEXT | NOT NULL | Tag label (e.g., "Italian", "breakfast") |
| confidence | REAL | NOT NULL | Confidence score 0.0–1.0 |
| user_override | INTEGER | NOT NULL DEFAULT 0 | 1 if user manually set this tag |

**Index**: `(recipe_id, domain)` for filtered retrieval.
**Unique**: `(recipe_id, domain, label)` prevents duplicate tags.

### change_log

Append-only log of all mutations. Used for sync export/import.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | Local sequence number |
| recipe_id | TEXT | NOT NULL | Which recipe was changed |
| field_name | TEXT | NOT NULL | Field that changed (e.g., "title", "notes", "__deleted", "ingredients", "tags") |
| field_value | TEXT | | New value (JSON-encoded for complex fields, plain text for simple) |
| modified_at | TEXT | NOT NULL | ISO 8601 UTC with microsecond precision |
| device_id | TEXT | NOT NULL | Which device made this change |
| synced | INTEGER | NOT NULL DEFAULT 0 | 0 = pending export, 1 = exported to iCloud |

**Index**: `(synced, id)` for efficient export of pending changes.
**Index**: `(recipe_id, field_name, modified_at)` for efficient merge lookups.

### sync_state

Tracks sync progress per remote device.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| device_id | TEXT | PRIMARY KEY | Remote device identifier |
| last_imported_id | INTEGER | NOT NULL DEFAULT 0 | Last change_log ID imported from this device |
| last_import_at | TEXT | | ISO 8601 UTC timestamp of last import |

### schema_version

Tracks database migration state.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| version | INTEGER | PRIMARY KEY | Migration version number |
| applied_at | TEXT | NOT NULL | When this migration was applied |

## State Transitions

### Recipe Lifecycle

```
[Not Exists] ──save──> [Active] ──delete──> [Soft Deleted]
                          │                       │
                          │<──restore (sync)───────┘
                          │
                          ├──edit──> [Active] (updated_at changes)
                          │
                          └──re-scrape (same URL)──> [Active] (fields updated)
```

### Change Log Entry Lifecycle

```
[Created, synced=0] ──export to iCloud──> [synced=1]
```

Change log entries are never modified after creation. They are append-only.

### Sync Flow

```
1. App writes to recipes table
2. Trigger/hook appends to change_log (synced=0)
3. Export: SELECT from change_log WHERE synced=0, write to iCloud file, SET synced=1
4. Import: Read remote device files, INSERT into change_log, merge into recipes
5. Merge: For each field, latest modified_at wins (LWW)
```

## Validation Rules

- `source_url` must be a valid normalized URL (validated by existing url_ingestion module)
- `extraction_source` must be one of: "json_ld", "microdata", "ai_fallback"
- `domain` in tags must be one of: "cuisine", "course", "diet"
- `confidence` must be in range [0.0, 1.0]
- `modified_at` timestamps must be ISO 8601 UTC format
- `device_id` must be a non-empty string (UUID recommended)
- `step_number` must be >= 1
- `position` must be >= 0

## Indexes Summary

| Table | Index | Purpose |
|-------|-------|---------|
| recipes | source_url (UNIQUE) | Dedup on re-scrape (FR-003) |
| recipes | title, notes | Full-text search support (FR-011) |
| ingredients | (recipe_id, position) | Ordered retrieval |
| instructions | (recipe_id, step_number) | Ordered retrieval |
| tags | (recipe_id, domain) | Filtered tag retrieval |
| tags | (recipe_id, domain, label) UNIQUE | Prevent duplicate tags |
| change_log | (synced, id) | Export pending changes |
| change_log | (recipe_id, field_name, modified_at) | Merge lookups |
