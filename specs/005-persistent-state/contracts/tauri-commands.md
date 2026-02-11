# Tauri Command Contracts: Persistent State

**Feature Branch**: `005-persistent-state`
**Date**: 2026-02-09

These contracts define the Tauri IPC commands exposed to the frontend. All commands follow the existing pattern: `#[tauri::command]` async functions returning `Result<T, E>`.

---

## Storage Commands

### save_recipe

Persists an extracted recipe with tags to local storage. If a recipe with the same source URL already exists, updates it (FR-003).

**Command**: `save_recipe`

**Parameters**:
```typescript
{
  recipe: ExtractedRecipe,  // From extract_recipe command
  tags: TagSet,             // From tag_recipe command
  source_url: string        // Normalized source URL
}
```

**Response (success)**:
```typescript
{
  id: string,           // UUID of saved/updated recipe
  created: boolean      // true if new, false if updated existing
}
```

**Response (error)**:
```typescript
{
  error_type: "storage",
  message: string
}
```

**Behavior**:
- If `source_url` matches an existing recipe: update all fields, bump `updated_at`
- If new: generate UUID, set `created_at` and `updated_at`
- Append change log entries for all fields
- Transaction: all-or-nothing write (FR-008)

---

### get_recipe

Retrieves a single recipe by ID with all related data.

**Command**: `get_recipe`

**Parameters**:
```typescript
{
  id: string  // Recipe UUID
}
```

**Response (success)**:
```typescript
{
  id: string,
  source_url: string,
  title: ExtractedField<string>,
  description: ExtractedField<string>,
  ingredients: Ingredient[],
  instructions: Instruction[],
  prep_time_minutes: ExtractedField<number>,
  cook_time_minutes: ExtractedField<number>,
  servings: ExtractedField<string>,
  images: ExtractedField<string[]>,
  nutrition: ExtractedField<NutritionInfo>,
  extraction_source: string,
  tags: TagSet,
  notes: string,
  created_at: string,
  updated_at: string
}
```

**Response (error)**:
```typescript
{
  error_type: "not_found" | "storage",
  message: string
}
```

---

### list_recipes

Lists all recipes with summary data for browsing (FR-002, FR-004).

**Command**: `list_recipes`

**Parameters**:
```typescript
{
  // No parameters — returns all non-deleted recipes
}
```

**Response (success)**:
```typescript
{
  recipes: Array<{
    id: string,
    source_url: string,
    title: string | null,
    description: string | null,
    prep_time_minutes: number | null,
    cook_time_minutes: number | null,
    tags: TagSet,
    created_at: string,
    updated_at: string
  }>
}
```

**Behavior**:
- Returns all recipes where `deleted = 0`
- Sorted by `updated_at` descending (most recent first)
- Includes tags for filtering in the UI
- Must complete in <100ms for 5,000 recipes (SC-002)

---

### search_recipes

Searches and filters recipes by title, ingredients, and tags (FR-011).

**Command**: `search_recipes`

**Parameters**:
```typescript
{
  query?: string,              // Free-text search in title and ingredients
  cuisine_tags?: string[],     // Filter by cuisine labels
  course_tags?: string[],      // Filter by course labels
  diet_tags?: string[]         // Filter by diet labels
}
```

**Response (success)**:
```typescript
{
  recipes: Array<{
    id: string,
    source_url: string,
    title: string | null,
    description: string | null,
    prep_time_minutes: number | null,
    cook_time_minutes: number | null,
    tags: TagSet,
    created_at: string,
    updated_at: string
  }>
}
```

**Behavior**:
- Free-text query searches title and ingredient names (case-insensitive)
- Tag filters are AND within a domain (e.g., cuisine = "Italian" AND "Mediterranean")
- Tag filters are AND across domains (e.g., cuisine = "Italian" AND course = "dinner")
- Returns only non-deleted recipes
- Must complete in <100ms for 5,000 recipes (SC-002)

---

### update_recipe

Updates user-editable fields of a saved recipe (FR-015, FR-016).

**Command**: `update_recipe`

**Parameters**:
```typescript
{
  id: string,
  fields: {
    title?: string,
    description?: string,
    ingredients?: Ingredient[],
    instructions?: Instruction[],
    prep_time_minutes?: number,
    cook_time_minutes?: number,
    servings?: string,
    notes?: string,
    tags?: TagSet
  }
}
```

**Response (success)**:
```typescript
{
  updated_at: string  // New updated_at timestamp
}
```

**Response (error)**:
```typescript
{
  error_type: "not_found" | "storage",
  message: string
}
```

**Behavior**:
- Only provided fields are updated (partial update)
- Appends change log entries for each changed field
- Updates `updated_at` timestamp
- Transaction: all-or-nothing write

---

### delete_recipe

Soft-deletes a recipe from the collection (FR-009).

**Command**: `delete_recipe`

**Parameters**:
```typescript
{
  id: string  // Recipe UUID
}
```

**Response (success)**:
```typescript
{
  deleted: boolean  // true if recipe was found and deleted
}
```

**Behavior**:
- Sets `deleted = 1` on the recipe row
- Appends `__deleted` change log entry
- Does NOT physically remove data (needed for sync)

---

## Export/Import Commands

### export_recipes

Exports recipes as schema.org/Recipe JSON (FR-006).

**Command**: `export_recipes`

**Parameters**:
```typescript
{
  recipe_ids?: string[],  // Specific IDs to export. If omitted, exports all.
  file_path: string       // Destination file path
}
```

**Response (success)**:
```typescript
{
  count: number,     // Number of recipes exported
  file_path: string  // Path where file was written
}
```

**Behavior**:
- Exports as JSON array of schema.org/Recipe objects
- Includes all recipe data, tags, and notes
- Works fully offline (FR-004)
- If `recipe_ids` is omitted, exports all non-deleted recipes

---

### import_recipes

Imports recipes from schema.org/Recipe JSON (FR-007).

**Command**: `import_recipes`

**Parameters**:
```typescript
{
  file_path: string  // Source file path
}
```

**Response (success)**:
```typescript
{
  imported: number,   // New recipes created
  updated: number,    // Existing recipes updated (matched by source URL)
  skipped: number,    // Recipes that couldn't be parsed
  errors: string[]    // Per-recipe error messages for skipped items
}
```

**Behavior**:
- Parses schema.org/Recipe JSON array
- Deduplicates by source URL (matching recipes are updated)
- Recipes without a source URL get a synthetic URL based on title hash
- Transaction per recipe (one failure doesn't block others)

---

### backup_collection

Creates a full backup of the database (FR-014).

**Command**: `backup_collection`

**Parameters**:
```typescript
{
  file_path: string  // Destination file path for backup
}
```

**Response (success)**:
```typescript
{
  file_path: string,
  recipe_count: number,
  size_bytes: number
}
```

**Behavior**:
- Uses SQLite backup API for atomic, consistent backup
- Includes all data (recipes, tags, change log, sync state)
- Works while database is in use (no lock required)

---

### restore_collection

Restores from a backup file (FR-014).

**Command**: `restore_collection`

**Parameters**:
```typescript
{
  file_path: string  // Source backup file path
}
```

**Response (success)**:
```typescript
{
  recipe_count: number  // Number of recipes in restored database
}
```

**Behavior**:
- Replaces the current database entirely
- Validates backup file integrity before replacing
- Requires app restart after restore

---

## Sync Commands

### trigger_sync

Manually triggers a sync cycle (export pending changes, import remote changes).

**Command**: `trigger_sync`

**Parameters**:
```typescript
{
  // No parameters
}
```

**Response (success)**:
```typescript
{
  exported: number,   // Change log entries exported
  imported: number,   // Change log entries imported from remote devices
  merged: number      // Recipes with fields merged from remote changes
}
```

**Behavior**:
- Exports pending change log entries to iCloud container
- Reads change files from all remote devices
- Merges using per-field LWW
- Non-blocking (runs on background thread)

---

### get_sync_status

Returns current sync state information.

**Command**: `get_sync_status`

**Parameters**:
```typescript
{
  // No parameters
}
```

**Response (success)**:
```typescript
{
  enabled: boolean,
  pending_changes: number,      // Unsynced local changes
  last_sync_at: string | null,  // ISO 8601 timestamp
  known_devices: Array<{
    device_id: string,
    last_imported_at: string | null
  }>
}
```
