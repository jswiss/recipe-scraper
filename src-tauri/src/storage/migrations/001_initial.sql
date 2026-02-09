-- Migration 001: Initial schema for persistent state
-- All tables, indexes, and constraints per data-model.md

CREATE TABLE IF NOT EXISTS recipes (
    id                      TEXT PRIMARY KEY,
    source_url              TEXT NOT NULL UNIQUE,
    title                   TEXT,
    title_status            TEXT NOT NULL,
    title_justification     TEXT,
    description             TEXT,
    description_status      TEXT NOT NULL,
    description_justification TEXT,
    servings                TEXT,
    servings_status         TEXT NOT NULL,
    servings_justification  TEXT,
    prep_time_minutes       INTEGER,
    prep_time_status        TEXT NOT NULL,
    prep_time_justification TEXT,
    cook_time_minutes       INTEGER,
    cook_time_status        TEXT NOT NULL,
    cook_time_justification TEXT,
    images_json             TEXT,
    images_status           TEXT NOT NULL,
    images_justification    TEXT,
    nutrition_json          TEXT,
    nutrition_status        TEXT NOT NULL,
    nutrition_justification TEXT,
    extraction_source       TEXT NOT NULL,
    notes                   TEXT NOT NULL DEFAULT '',
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL,
    deleted                 INTEGER NOT NULL DEFAULT 0,
    device_id               TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ingredients (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id   TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    name        TEXT NOT NULL,
    quantity    REAL,
    unit        TEXT,
    raw_text    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ingredients_recipe_position
    ON ingredients(recipe_id, position);

CREATE TABLE IF NOT EXISTS instructions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id   TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    text        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_instructions_recipe_step
    ON instructions(recipe_id, step_number);

CREATE TABLE IF NOT EXISTS tags (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id     TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    domain        TEXT NOT NULL,
    label         TEXT NOT NULL,
    confidence    REAL NOT NULL,
    user_override INTEGER NOT NULL DEFAULT 0,
    UNIQUE(recipe_id, domain, label)
);

CREATE INDEX IF NOT EXISTS idx_tags_recipe_domain
    ON tags(recipe_id, domain);

CREATE TABLE IF NOT EXISTS change_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id   TEXT NOT NULL,
    field_name  TEXT NOT NULL,
    field_value TEXT,
    modified_at TEXT NOT NULL,
    device_id   TEXT NOT NULL,
    synced      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_change_log_synced_id
    ON change_log(synced, id);

CREATE INDEX IF NOT EXISTS idx_change_log_recipe_field_time
    ON change_log(recipe_id, field_name, modified_at);

CREATE TABLE IF NOT EXISTS sync_state (
    device_id        TEXT PRIMARY KEY,
    last_imported_id INTEGER NOT NULL DEFAULT 0,
    last_import_at   TEXT
);
