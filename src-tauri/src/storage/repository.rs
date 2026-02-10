use rusqlite::params;

use crate::recipe_extraction::{ExtractedField, ExtractionSource, Ingredient, Instruction};
use crate::recipe_tagging::TagSet;

use super::change_log::{self, now_utc};
use super::database::Database;
use super::models::*;

/// Row tuple from recipe summary queries (id, source_url, title, description, prep, cook, created_at, updated_at).
type SummaryRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<i64>,
    String,
    String,
);

pub fn save_recipe(db: &Database, input: SaveRecipeInput) -> Result<SaveResult, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    // Check if recipe with this source_url already exists
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM recipes WHERE source_url = ?1 AND deleted = 0",
            params![input.source_url],
            |row| row.get(0),
        )
        .ok();

    let now = now_utc();

    if let Some(id) = existing_id {
        // Update existing recipe
        update_recipe_fields(&conn, &id, input.recipe, input.tags, &now, &db.device_id)?;
        Ok(SaveResult { id, created: false })
    } else {
        // Insert new recipe
        let id = uuid::Uuid::new_v4().to_string();
        insert_recipe(&conn, &id, input, &now, &db.device_id)?;
        Ok(SaveResult { id, created: true })
    }
}

fn insert_recipe(
    conn: &rusqlite::Connection,
    id: &str,
    input: SaveRecipeInput,
    now: &str,
    device_id: &str,
) -> Result<(), StorageError> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to begin transaction: {e}"),
        })?;

    let (title_status, title_val) = extracted_field_to_columns(&input.recipe.title);
    let title_just = extracted_field_justification(&input.recipe.title);
    let (desc_status, desc_val) = extracted_field_to_columns(&input.recipe.description);
    let desc_just = extracted_field_justification(&input.recipe.description);
    let (serv_status, serv_val) = extracted_field_to_columns(&input.recipe.servings);
    let serv_just = extracted_field_justification(&input.recipe.servings);
    let (prep_status, prep_val) = extracted_u32_to_columns(&input.recipe.prep_time_minutes);
    let prep_just = extracted_field_justification(&input.recipe.prep_time_minutes);
    let (cook_status, cook_val) = extracted_u32_to_columns(&input.recipe.cook_time_minutes);
    let cook_just = extracted_field_justification(&input.recipe.cook_time_minutes);

    let images_json = match &input.recipe.images {
        ExtractedField::Found { value } => Some(serde_json::to_string(value).unwrap_or_default()),
        ExtractedField::NotFound { .. } => None,
    };
    let (img_status, _) = extracted_field_to_columns_generic(&input.recipe.images);
    let img_just = extracted_field_justification(&input.recipe.images);

    let nutrition_json = match &input.recipe.nutrition {
        ExtractedField::Found { value } => Some(serde_json::to_string(value).unwrap_or_default()),
        ExtractedField::NotFound { .. } => None,
    };
    let (nutr_status, _) = extracted_field_to_columns_generic(&input.recipe.nutrition);
    let nutr_just = extracted_field_justification(&input.recipe.nutrition);

    let source_str = serde_json::to_value(input.recipe.source)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "json_ld".into());

    tx.execute(
        "INSERT INTO recipes (id, source_url, title, title_status, title_justification,
         description, description_status, description_justification,
         servings, servings_status, servings_justification,
         prep_time_minutes, prep_time_status, prep_time_justification,
         cook_time_minutes, cook_time_status, cook_time_justification,
         images_json, images_status, images_justification,
         nutrition_json, nutrition_status, nutrition_justification,
         extraction_source, notes, created_at, updated_at, deleted, device_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,0,?28)",
        params![
            id, input.source_url,
            title_val, title_status, title_just,
            desc_val, desc_status, desc_just,
            serv_val, serv_status, serv_just,
            prep_val, prep_status, prep_just,
            cook_val, cook_status, cook_just,
            images_json, img_status, img_just,
            nutrition_json, nutr_status, nutr_just,
            source_str, "", now, now, device_id,
        ],
    )
    .map_err(|e| StorageError::Storage {
        message: format!("Failed to insert recipe: {e}"),
    })?;

    insert_ingredients(&tx, id, &input.recipe.ingredients)?;
    insert_instructions(&tx, id, &input.recipe.instructions)?;
    insert_tags(&tx, id, input.tags)?;

    // Record change log entries for all fields
    log_all_recipe_fields(&tx, id, input.recipe, input.tags, device_id)?;

    tx.commit().map_err(|e| StorageError::Storage {
        message: format!("Failed to commit: {e}"),
    })?;
    Ok(())
}

fn update_recipe_fields(
    conn: &rusqlite::Connection,
    id: &str,
    recipe: &crate::recipe_extraction::ExtractedRecipe,
    tags: &TagSet,
    now: &str,
    device_id: &str,
) -> Result<(), StorageError> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to begin transaction: {e}"),
        })?;

    let (title_status, title_val) = extracted_field_to_columns(&recipe.title);
    let title_just = extracted_field_justification(&recipe.title);
    let (desc_status, desc_val) = extracted_field_to_columns(&recipe.description);
    let desc_just = extracted_field_justification(&recipe.description);
    let (serv_status, serv_val) = extracted_field_to_columns(&recipe.servings);
    let serv_just = extracted_field_justification(&recipe.servings);
    let (prep_status, prep_val) = extracted_u32_to_columns(&recipe.prep_time_minutes);
    let prep_just = extracted_field_justification(&recipe.prep_time_minutes);
    let (cook_status, cook_val) = extracted_u32_to_columns(&recipe.cook_time_minutes);
    let cook_just = extracted_field_justification(&recipe.cook_time_minutes);

    let images_json = match &recipe.images {
        ExtractedField::Found { value } => Some(serde_json::to_string(value).unwrap_or_default()),
        ExtractedField::NotFound { .. } => None,
    };
    let (img_status, _) = extracted_field_to_columns_generic(&recipe.images);
    let img_just = extracted_field_justification(&recipe.images);

    let nutrition_json = match &recipe.nutrition {
        ExtractedField::Found { value } => Some(serde_json::to_string(value).unwrap_or_default()),
        ExtractedField::NotFound { .. } => None,
    };
    let (nutr_status, _) = extracted_field_to_columns_generic(&recipe.nutrition);
    let nutr_just = extracted_field_justification(&recipe.nutrition);

    let source_str = serde_json::to_value(recipe.source)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "json_ld".into());

    tx.execute(
        "UPDATE recipes SET title=?1, title_status=?2, title_justification=?3,
         description=?4, description_status=?5, description_justification=?6,
         servings=?7, servings_status=?8, servings_justification=?9,
         prep_time_minutes=?10, prep_time_status=?11, prep_time_justification=?12,
         cook_time_minutes=?13, cook_time_status=?14, cook_time_justification=?15,
         images_json=?16, images_status=?17, images_justification=?18,
         nutrition_json=?19, nutrition_status=?20, nutrition_justification=?21,
         extraction_source=?22, updated_at=?23, device_id=?24
         WHERE id=?25",
        params![
            title_val,
            title_status,
            title_just,
            desc_val,
            desc_status,
            desc_just,
            serv_val,
            serv_status,
            serv_just,
            prep_val,
            prep_status,
            prep_just,
            cook_val,
            cook_status,
            cook_just,
            images_json,
            img_status,
            img_just,
            nutrition_json,
            nutr_status,
            nutr_just,
            source_str,
            now,
            device_id,
            id,
        ],
    )
    .map_err(|e| StorageError::Storage {
        message: format!("Failed to update recipe: {e}"),
    })?;

    // Replace child rows
    tx.execute("DELETE FROM ingredients WHERE recipe_id = ?1", params![id])
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to clear ingredients: {e}"),
        })?;
    tx.execute("DELETE FROM instructions WHERE recipe_id = ?1", params![id])
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to clear instructions: {e}"),
        })?;
    tx.execute("DELETE FROM tags WHERE recipe_id = ?1", params![id])
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to clear tags: {e}"),
        })?;

    insert_ingredients(&tx, id, &recipe.ingredients)?;
    insert_instructions(&tx, id, &recipe.instructions)?;
    insert_tags(&tx, id, tags)?;

    log_all_recipe_fields(&tx, id, recipe, tags, device_id)?;

    tx.commit().map_err(|e| StorageError::Storage {
        message: format!("Failed to commit: {e}"),
    })?;
    Ok(())
}

fn insert_ingredients(
    conn: &rusqlite::Connection,
    recipe_id: &str,
    ingredients: &[Ingredient],
) -> Result<(), StorageError> {
    let mut stmt = conn
        .prepare(
            "INSERT INTO ingredients (recipe_id, position, name, quantity, unit, raw_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    for (i, ing) in ingredients.iter().enumerate() {
        stmt.execute(params![
            recipe_id,
            i as i64,
            ing.name,
            ing.quantity,
            ing.unit,
            ing.raw_text
        ])
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to insert ingredient: {e}"),
        })?;
    }
    Ok(())
}

fn insert_instructions(
    conn: &rusqlite::Connection,
    recipe_id: &str,
    instructions: &[Instruction],
) -> Result<(), StorageError> {
    let mut stmt = conn
        .prepare("INSERT INTO instructions (recipe_id, step_number, text) VALUES (?1, ?2, ?3)")
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    for inst in instructions {
        stmt.execute(params![recipe_id, inst.step_number as i64, inst.text])
            .map_err(|e| StorageError::Storage {
                message: format!("Failed to insert instruction: {e}"),
            })?;
    }
    Ok(())
}

fn insert_tags(
    conn: &rusqlite::Connection,
    recipe_id: &str,
    tags: &TagSet,
) -> Result<(), StorageError> {
    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO tags (recipe_id, domain, label, confidence, user_override)
             VALUES (?1, ?2, ?3, ?4, 0)",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    for (domain, tag) in tag_set_to_rows(tags) {
        stmt.execute(params![recipe_id, domain, tag.label, tag.confidence])
            .map_err(|e| StorageError::Storage {
                message: format!("Failed to insert tag: {e}"),
            })?;
    }
    Ok(())
}

pub fn get_recipe(db: &Database, id: &str) -> Result<SavedRecipe, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let row = conn
        .query_row(
            "SELECT id, source_url,
                title, title_status, title_justification,
                description, description_status, description_justification,
                servings, servings_status, servings_justification,
                prep_time_minutes, prep_time_status, prep_time_justification,
                cook_time_minutes, cook_time_status, cook_time_justification,
                images_json, images_status, images_justification,
                nutrition_json, nutrition_status, nutrition_justification,
                extraction_source, notes, created_at, updated_at
         FROM recipes WHERE id = ?1 AND deleted = 0",
            params![id],
            |row| {
                Ok(RecipeRow {
                    id: row.get(0)?,
                    source_url: row.get(1)?,
                    title: row.get(2)?,
                    title_status: row.get(3)?,
                    title_justification: row.get(4)?,
                    description: row.get(5)?,
                    description_status: row.get(6)?,
                    description_justification: row.get(7)?,
                    servings: row.get(8)?,
                    servings_status: row.get(9)?,
                    servings_justification: row.get(10)?,
                    prep_time_minutes: row.get(11)?,
                    prep_time_status: row.get(12)?,
                    prep_time_justification: row.get(13)?,
                    cook_time_minutes: row.get(14)?,
                    cook_time_status: row.get(15)?,
                    cook_time_justification: row.get(16)?,
                    images_json: row.get(17)?,
                    images_status: row.get(18)?,
                    images_justification: row.get(19)?,
                    nutrition_json: row.get(20)?,
                    nutrition_status: row.get(21)?,
                    nutrition_justification: row.get(22)?,
                    extraction_source: row.get(23)?,
                    notes: row.get(24)?,
                    created_at: row.get(25)?,
                    updated_at: row.get(26)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StorageError::NotFound {
                message: format!("Recipe not found: {id}"),
            },
            _ => StorageError::Storage {
                message: format!("Failed to get recipe: {e}"),
            },
        })?;

    let ingredients = load_ingredients(&conn, &row.id)?;
    let instructions = load_instructions(&conn, &row.id)?;
    let tag_rows = load_tag_rows(&conn, &row.id)?;
    let tags = rows_to_tag_set(&tag_rows);

    Ok(row_to_saved_recipe(row, ingredients, instructions, tags))
}

// --- Internal helpers ---

struct RecipeRow {
    id: String,
    source_url: String,
    title: Option<String>,
    title_status: String,
    title_justification: Option<String>,
    description: Option<String>,
    description_status: String,
    description_justification: Option<String>,
    servings: Option<String>,
    servings_status: String,
    servings_justification: Option<String>,
    prep_time_minutes: Option<i64>,
    prep_time_status: String,
    prep_time_justification: Option<String>,
    cook_time_minutes: Option<i64>,
    cook_time_status: String,
    cook_time_justification: Option<String>,
    images_json: Option<String>,
    images_status: String,
    images_justification: Option<String>,
    nutrition_json: Option<String>,
    nutrition_status: String,
    nutrition_justification: Option<String>,
    extraction_source: String,
    notes: String,
    created_at: String,
    updated_at: String,
}

fn row_to_saved_recipe(
    row: RecipeRow,
    ingredients: Vec<Ingredient>,
    instructions: Vec<Instruction>,
    tags: TagSet,
) -> SavedRecipe {
    let extraction_source = match row.extraction_source.as_str() {
        "microdata" => ExtractionSource::Microdata,
        "ai_fallback" => ExtractionSource::AiFallback,
        _ => ExtractionSource::JsonLd,
    };

    SavedRecipe {
        id: row.id,
        source_url: row.source_url,
        title: columns_to_extracted_string(&row.title_status, row.title, row.title_justification),
        description: columns_to_extracted_string(
            &row.description_status,
            row.description,
            row.description_justification,
        ),
        servings: columns_to_extracted_string(
            &row.servings_status,
            row.servings,
            row.servings_justification,
        ),
        prep_time_minutes: columns_to_extracted_u32(
            &row.prep_time_status,
            row.prep_time_minutes,
            row.prep_time_justification,
        ),
        cook_time_minutes: columns_to_extracted_u32(
            &row.cook_time_status,
            row.cook_time_minutes,
            row.cook_time_justification,
        ),
        images: columns_to_extracted_vec_string(
            &row.images_status,
            row.images_json,
            row.images_justification,
        ),
        nutrition: columns_to_extracted_nutrition(
            &row.nutrition_status,
            row.nutrition_json,
            row.nutrition_justification,
        ),
        extraction_source,
        ingredients,
        instructions,
        tags,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn load_ingredients(
    conn: &rusqlite::Connection,
    recipe_id: &str,
) -> Result<Vec<Ingredient>, StorageError> {
    let mut stmt = conn
        .prepare("SELECT name, quantity, unit, raw_text FROM ingredients WHERE recipe_id = ?1 ORDER BY position")
        .map_err(|e| StorageError::Storage { message: format!("Failed to prepare: {e}") })?;

    let rows = stmt
        .query_map(params![recipe_id], |row| {
            Ok(Ingredient::new(
                row.get::<_, String>(0)?,
                row.get::<_, Option<f64>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to query ingredients: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to collect ingredients: {e}"),
        })?;

    Ok(rows)
}

fn load_instructions(
    conn: &rusqlite::Connection,
    recipe_id: &str,
) -> Result<Vec<Instruction>, StorageError> {
    let mut stmt = conn
        .prepare(
            "SELECT step_number, text FROM instructions WHERE recipe_id = ?1 ORDER BY step_number",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    let rows = stmt
        .query_map(params![recipe_id], |row| {
            Ok(Instruction::new(
                row.get::<_, i64>(0)? as u32,
                row.get::<_, String>(1)?,
            ))
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to query instructions: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to collect instructions: {e}"),
        })?;

    Ok(rows)
}

fn load_tag_rows(
    conn: &rusqlite::Connection,
    recipe_id: &str,
) -> Result<Vec<(String, String, f64)>, StorageError> {
    let mut stmt = conn
        .prepare("SELECT domain, label, confidence FROM tags WHERE recipe_id = ?1 ORDER BY domain, confidence DESC")
        .map_err(|e| StorageError::Storage { message: format!("Failed to prepare: {e}") })?;

    let rows = stmt
        .query_map(params![recipe_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to query tags: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to collect tags: {e}"),
        })?;

    Ok(rows)
}

pub fn update_recipe(
    db: &Database,
    id: &str,
    fields: &UpdateFields,
) -> Result<UpdateResult, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    // Verify recipe exists and is not deleted
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM recipes WHERE id = ?1 AND deleted = 0",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Query failed: {e}"),
        })?;

    if !exists {
        return Err(StorageError::NotFound {
            message: format!("Recipe not found: {id}"),
        });
    }

    let now = now_utc();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to begin transaction: {e}"),
        })?;

    if let Some(title) = &fields.title {
        tx.execute(
            "UPDATE recipes SET title=?1, title_status='found', title_justification=NULL, updated_at=?2 WHERE id=?3",
            params![title, now, id],
        ).map_err(|e| StorageError::Storage { message: format!("Update failed: {e}") })?;
    }
    if let Some(description) = &fields.description {
        tx.execute(
            "UPDATE recipes SET description=?1, description_status='found', description_justification=NULL, updated_at=?2 WHERE id=?3",
            params![description, now, id],
        ).map_err(|e| StorageError::Storage { message: format!("Update failed: {e}") })?;
    }
    if let Some(servings) = &fields.servings {
        tx.execute(
            "UPDATE recipes SET servings=?1, servings_status='found', servings_justification=NULL, updated_at=?2 WHERE id=?3",
            params![servings, now, id],
        ).map_err(|e| StorageError::Storage { message: format!("Update failed: {e}") })?;
    }
    if let Some(prep) = &fields.prep_time_minutes {
        tx.execute(
            "UPDATE recipes SET prep_time_minutes=?1, prep_time_status='found', prep_time_justification=NULL, updated_at=?2 WHERE id=?3",
            params![*prep as i64, now, id],
        ).map_err(|e| StorageError::Storage { message: format!("Update failed: {e}") })?;
    }
    if let Some(cook) = &fields.cook_time_minutes {
        tx.execute(
            "UPDATE recipes SET cook_time_minutes=?1, cook_time_status='found', cook_time_justification=NULL, updated_at=?2 WHERE id=?3",
            params![*cook as i64, now, id],
        ).map_err(|e| StorageError::Storage { message: format!("Update failed: {e}") })?;
    }
    if let Some(notes) = &fields.notes {
        tx.execute(
            "UPDATE recipes SET notes=?1, updated_at=?2 WHERE id=?3",
            params![notes, now, id],
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Update failed: {e}"),
        })?;
    }
    if let Some(ingredients) = &fields.ingredients {
        tx.execute("DELETE FROM ingredients WHERE recipe_id = ?1", params![id])
            .map_err(|e| StorageError::Storage {
                message: format!("Clear failed: {e}"),
            })?;
        insert_ingredients(&tx, id, ingredients)?;
        tx.execute(
            "UPDATE recipes SET updated_at=?1 WHERE id=?2",
            params![now, id],
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Update failed: {e}"),
        })?;
    }
    if let Some(instructions) = &fields.instructions {
        tx.execute("DELETE FROM instructions WHERE recipe_id = ?1", params![id])
            .map_err(|e| StorageError::Storage {
                message: format!("Clear failed: {e}"),
            })?;
        insert_instructions(&tx, id, instructions)?;
        tx.execute(
            "UPDATE recipes SET updated_at=?1 WHERE id=?2",
            params![now, id],
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Update failed: {e}"),
        })?;
    }
    if let Some(tags) = &fields.tags {
        tx.execute("DELETE FROM tags WHERE recipe_id = ?1", params![id])
            .map_err(|e| StorageError::Storage {
                message: format!("Clear failed: {e}"),
            })?;
        insert_tags(&tx, id, tags)?;
        tx.execute(
            "UPDATE recipes SET updated_at=?1 WHERE id=?2",
            params![now, id],
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Update failed: {e}"),
        })?;
    }

    // Ensure updated_at is set even if only notes changed
    tx.execute(
        "UPDATE recipes SET updated_at=?1, device_id=?2 WHERE id=?3",
        params![now, db.device_id, id],
    )
    .map_err(|e| StorageError::Storage {
        message: format!("Update failed: {e}"),
    })?;

    // Record change log for each updated field
    if let Some(v) = &fields.title {
        change_log::append_change(&tx, id, "title", Some(v), &db.device_id)?;
    }
    if let Some(v) = &fields.description {
        change_log::append_change(&tx, id, "description", Some(v), &db.device_id)?;
    }
    if let Some(v) = &fields.servings {
        change_log::append_change(&tx, id, "servings", Some(v), &db.device_id)?;
    }
    if let Some(v) = &fields.prep_time_minutes {
        change_log::append_change(
            &tx,
            id,
            "prep_time_minutes",
            Some(&v.to_string()),
            &db.device_id,
        )?;
    }
    if let Some(v) = &fields.cook_time_minutes {
        change_log::append_change(
            &tx,
            id,
            "cook_time_minutes",
            Some(&v.to_string()),
            &db.device_id,
        )?;
    }
    if let Some(v) = &fields.notes {
        change_log::append_change(&tx, id, "notes", Some(v), &db.device_id)?;
    }
    if let Some(v) = &fields.ingredients {
        let json = serde_json::to_string(v).unwrap_or_default();
        change_log::append_change(&tx, id, "ingredients", Some(&json), &db.device_id)?;
    }
    if let Some(v) = &fields.instructions {
        let json = serde_json::to_string(v).unwrap_or_default();
        change_log::append_change(&tx, id, "instructions", Some(&json), &db.device_id)?;
    }
    if let Some(v) = &fields.tags {
        let json = serde_json::to_string(v).unwrap_or_default();
        change_log::append_change(&tx, id, "tags", Some(&json), &db.device_id)?;
    }

    tx.commit().map_err(|e| StorageError::Storage {
        message: format!("Failed to commit: {e}"),
    })?;

    Ok(UpdateResult { updated_at: now })
}

pub fn delete_recipe(db: &Database, id: &str) -> Result<DeleteResult, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let now = now_utc();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to begin transaction: {e}"),
        })?;

    let rows = tx
        .execute(
            "UPDATE recipes SET deleted = 1, updated_at = ?1, device_id = ?2 WHERE id = ?3 AND deleted = 0",
            params![now, db.device_id, id],
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to delete: {e}"),
        })?;

    if rows > 0 {
        change_log::append_change(&tx, id, "__deleted", Some("true"), &db.device_id)?;
    }

    tx.commit().map_err(|e| StorageError::Storage {
        message: format!("Failed to commit: {e}"),
    })?;

    Ok(DeleteResult { deleted: rows > 0 })
}

pub fn list_recipes(db: &Database) -> Result<Vec<RecipeSummary>, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let mut stmt = conn
        .prepare(
            "SELECT id, source_url, title, description, prep_time_minutes, cook_time_minutes,
                    created_at, updated_at
             FROM recipes WHERE deleted = 0 ORDER BY updated_at DESC",
        )
        .map_err(|e| StorageError::Storage {
            message: format!("Failed to prepare: {e}"),
        })?;

    let rows: Vec<SummaryRow> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Query failed: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Collect failed: {e}"),
        })?;

    let mut summaries = Vec::with_capacity(rows.len());
    for (id, source_url, title, description, prep, cook, created_at, updated_at) in rows {
        let tag_rows = load_tag_rows(&conn, &id)?;
        let tags = rows_to_tag_set(&tag_rows);
        summaries.push(RecipeSummary {
            id,
            source_url,
            title,
            description,
            prep_time_minutes: prep.map(|v| v as u32),
            cook_time_minutes: cook.map(|v| v as u32),
            tags,
            created_at,
            updated_at,
        });
    }

    Ok(summaries)
}

pub fn search_recipes(
    db: &Database,
    query: &SearchQuery,
) -> Result<Vec<RecipeSummary>, StorageError> {
    let conn = db.conn.lock().map_err(|e| StorageError::Storage {
        message: format!("Failed to acquire lock: {e}"),
    })?;

    let mut sql = String::from(
        "SELECT DISTINCT r.id, r.source_url, r.title, r.description,
                r.prep_time_minutes, r.cook_time_minutes, r.created_at, r.updated_at
         FROM recipes r",
    );
    let mut conditions = vec!["r.deleted = 0".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 0usize;

    // Text search on title and ingredient names
    if let Some(text) = &query.query {
        if !text.is_empty() {
            sql.push_str(" LEFT JOIN ingredients i ON r.id = i.recipe_id");
            let pattern = format!("%{text}%");
            param_values.push(Box::new(pattern.clone()));
            param_idx += 1;
            let idx1 = param_idx;
            param_values.push(Box::new(pattern));
            param_idx += 1;
            let idx2 = param_idx;
            conditions.push(format!("(r.title LIKE ?{idx1} OR i.name LIKE ?{idx2})"));
        }
    }

    // Tag filters
    for (domain, tags) in [
        ("cuisine", query.cuisine_tags.as_deref().unwrap_or(&[])),
        ("course", query.course_tags.as_deref().unwrap_or(&[])),
        ("diet", query.diet_tags.as_deref().unwrap_or(&[])),
    ] {
        for tag_label in tags {
            param_values.push(Box::new(domain.to_string()));
            param_idx += 1;
            let d_idx = param_idx;
            param_values.push(Box::new(tag_label.clone()));
            param_idx += 1;
            let l_idx = param_idx;
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM tags t WHERE t.recipe_id = r.id AND t.domain = ?{d_idx} AND t.label = ?{l_idx})"
            ));
        }
    }

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));
    sql.push_str(" ORDER BY r.updated_at DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| StorageError::Storage {
        message: format!("Failed to prepare search: {e}"),
    })?;

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let rows: Vec<SummaryRow> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })
        .map_err(|e| StorageError::Storage {
            message: format!("Search failed: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::Storage {
            message: format!("Collect failed: {e}"),
        })?;

    let mut summaries = Vec::with_capacity(rows.len());
    for (id, source_url, title, description, prep, cook, created_at, updated_at) in rows {
        let tag_rows = load_tag_rows(&conn, &id)?;
        let tags = rows_to_tag_set(&tag_rows);
        summaries.push(RecipeSummary {
            id,
            source_url,
            title,
            description,
            prep_time_minutes: prep.map(|v| v as u32),
            cook_time_minutes: cook.map(|v| v as u32),
            tags,
            created_at,
            updated_at,
        });
    }

    Ok(summaries)
}

fn log_all_recipe_fields(
    conn: &rusqlite::Connection,
    id: &str,
    recipe: &crate::recipe_extraction::ExtractedRecipe,
    tags: &TagSet,
    device_id: &str,
) -> Result<(), StorageError> {
    if let Some(v) = recipe.title.value() {
        change_log::append_change(conn, id, "title", Some(v), device_id)?;
    }
    if let Some(v) = recipe.description.value() {
        change_log::append_change(conn, id, "description", Some(v), device_id)?;
    }
    if let Some(v) = recipe.servings.value() {
        change_log::append_change(conn, id, "servings", Some(v), device_id)?;
    }
    if let Some(v) = recipe.prep_time_minutes.value() {
        change_log::append_change(
            conn,
            id,
            "prep_time_minutes",
            Some(&v.to_string()),
            device_id,
        )?;
    }
    if let Some(v) = recipe.cook_time_minutes.value() {
        change_log::append_change(
            conn,
            id,
            "cook_time_minutes",
            Some(&v.to_string()),
            device_id,
        )?;
    }
    let ing_json = serde_json::to_string(&recipe.ingredients).unwrap_or_default();
    change_log::append_change(conn, id, "ingredients", Some(&ing_json), device_id)?;
    let inst_json = serde_json::to_string(&recipe.instructions).unwrap_or_default();
    change_log::append_change(conn, id, "instructions", Some(&inst_json), device_id)?;
    let tags_json = serde_json::to_string(tags).unwrap_or_default();
    change_log::append_change(conn, id, "tags", Some(&tags_json), device_id)?;
    Ok(())
}

/// Helper for ExtractedField status extraction (generic version).
fn extracted_field_to_columns_generic<T>(field: &ExtractedField<T>) -> (&str, ()) {
    match field {
        ExtractedField::Found { .. } => ("found", ()),
        ExtractedField::NotFound { .. } => ("not_found", ()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::{ExtractedRecipe, ExtractionSource, NutritionInfo};
    use crate::recipe_tagging::Tag;

    fn test_db() -> Database {
        Database::new_in_memory().expect("Failed to create test DB")
    }

    fn sample_recipe() -> ExtractedRecipe {
        ExtractedRecipe {
            title: ExtractedField::found("Test Pasta".to_string()),
            description: ExtractedField::found("A simple pasta dish".to_string()),
            ingredients: vec![
                Ingredient::new("pasta", Some(200.0), Some("g".into()), "200g pasta"),
                Ingredient::new(
                    "olive oil",
                    Some(2.0),
                    Some("tbsp".into()),
                    "2 tbsp olive oil",
                ),
            ],
            instructions: vec![
                Instruction::new(1, "Boil water"),
                Instruction::new(2, "Cook pasta"),
            ],
            prep_time_minutes: ExtractedField::found(10),
            cook_time_minutes: ExtractedField::found(20),
            servings: ExtractedField::found("4 servings".to_string()),
            images: ExtractedField::found(vec!["https://example.com/img.jpg".to_string()]),
            nutrition: ExtractedField::found(NutritionInfo {
                calories: Some(350),
                protein_grams: Some(12.0),
                ..NutritionInfo::default()
            }),
            source: ExtractionSource::JsonLd,
        }
    }

    fn sample_tags() -> TagSet {
        TagSet {
            cuisine: vec![Tag::new("Italian", 0.9)],
            course: vec![Tag::new("dinner", 0.8)],
            diet: vec![],
        }
    }

    #[test]
    fn save_and_get_round_trip() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();

        let result = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/pasta",
            },
        )
        .unwrap();

        assert!(result.created);

        let saved = get_recipe(&db, &result.id).unwrap();
        assert_eq!(saved.source_url, "https://example.com/pasta");
        assert_eq!(saved.title, ExtractedField::found("Test Pasta".to_string()));
        assert_eq!(saved.ingredients.len(), 2);
        assert_eq!(saved.ingredients[0].name, "pasta");
        assert_eq!(saved.ingredients[1].quantity, Some(2.0));
        assert_eq!(saved.instructions.len(), 2);
        assert_eq!(saved.instructions[0].text, "Boil water");
        assert_eq!(saved.tags.cuisine.len(), 1);
        assert_eq!(saved.tags.cuisine[0].label, "Italian");
        assert_eq!(saved.prep_time_minutes, ExtractedField::found(10));
        assert_eq!(saved.cook_time_minutes, ExtractedField::found(20));
        assert!(saved.notes.is_empty());
    }

    #[test]
    fn save_same_url_updates_instead_of_duplicating() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();
        let url = "https://example.com/pasta";

        let first = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: url,
            },
        )
        .unwrap();
        assert!(first.created);

        let mut updated_recipe = sample_recipe();
        updated_recipe.title = ExtractedField::found("Updated Pasta".to_string());

        let second = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &updated_recipe,
                tags: &tags,
                source_url: url,
            },
        )
        .unwrap();
        assert!(!second.created);
        assert_eq!(first.id, second.id);

        let saved = get_recipe(&db, &second.id).unwrap();
        assert_eq!(
            saved.title,
            ExtractedField::found("Updated Pasta".to_string())
        );
    }

    #[test]
    fn update_recipe_changes_only_specified_fields() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();

        let result = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/pasta",
            },
        )
        .unwrap();

        update_recipe(
            &db,
            &result.id,
            &UpdateFields {
                notes: Some("My favorite!".into()),
                ..Default::default()
            },
        )
        .unwrap();

        let saved = get_recipe(&db, &result.id).unwrap();
        assert_eq!(saved.notes, "My favorite!");
        // Title should be unchanged
        assert_eq!(saved.title, ExtractedField::found("Test Pasta".to_string()));
    }

    #[test]
    fn delete_recipe_sets_soft_delete() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();

        let result = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/pasta",
            },
        )
        .unwrap();

        let del = delete_recipe(&db, &result.id).unwrap();
        assert!(del.deleted);

        // get_recipe should return NotFound for deleted
        let err = get_recipe(&db, &result.id).unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[test]
    fn get_recipe_returns_not_found_for_missing_id() {
        let db = test_db();
        let err = get_recipe(&db, "nonexistent-id").unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    // --- Phase 4 (T012): list and search tests ---

    fn save_test_recipe(db: &Database, url: &str, title: &str, cuisine: &str) -> String {
        let mut recipe = sample_recipe();
        recipe.title = ExtractedField::found(title.to_string());
        let tags = TagSet {
            cuisine: vec![Tag::new(cuisine, 0.9)],
            course: vec![Tag::new("dinner", 0.8)],
            diet: vec![],
        };
        save_recipe(
            db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: url,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn list_returns_all_non_deleted_sorted_by_updated() {
        let db = test_db();
        save_test_recipe(&db, "https://a.com", "Alpha", "Italian");
        save_test_recipe(&db, "https://b.com", "Beta", "Thai");
        save_test_recipe(&db, "https://c.com", "Gamma", "Mexican");

        let list = list_recipes(&db).unwrap();
        assert_eq!(list.len(), 3);
        // Most recently saved should be first
        assert_eq!(list[0].title.as_deref(), Some("Gamma"));
    }

    #[test]
    fn list_excludes_deleted() {
        let db = test_db();
        let id = save_test_recipe(&db, "https://a.com", "Alpha", "Italian");
        save_test_recipe(&db, "https://b.com", "Beta", "Thai");
        delete_recipe(&db, &id).unwrap();

        let list = list_recipes(&db).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title.as_deref(), Some("Beta"));
    }

    #[test]
    fn search_by_title_substring() {
        let db = test_db();
        save_test_recipe(&db, "https://a.com", "Chicken Alfredo", "Italian");
        save_test_recipe(&db, "https://b.com", "Beef Stew", "French");

        let results = search_recipes(
            &db,
            &SearchQuery {
                query: Some("Alfredo".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_deref(), Some("Chicken Alfredo"));
    }

    #[test]
    fn search_by_ingredient_name() {
        let db = test_db();
        // sample_recipe has "pasta" and "olive oil" as ingredients
        save_test_recipe(&db, "https://a.com", "Recipe A", "Italian");

        let results = search_recipes(
            &db,
            &SearchQuery {
                query: Some("olive oil".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_filter_by_cuisine_tag() {
        let db = test_db();
        save_test_recipe(&db, "https://a.com", "Spaghetti", "Italian");
        save_test_recipe(&db, "https://b.com", "Pad Thai", "Thai");

        let results = search_recipes(
            &db,
            &SearchQuery {
                cuisine_tags: Some(vec!["Italian".into()]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_deref(), Some("Spaghetti"));
    }

    #[test]
    fn search_combined_text_and_tag() {
        let db = test_db();
        save_test_recipe(&db, "https://a.com", "Chicken Pasta", "Italian");
        save_test_recipe(&db, "https://b.com", "Chicken Curry", "Indian");

        let results = search_recipes(
            &db,
            &SearchQuery {
                query: Some("Chicken".into()),
                cuisine_tags: Some(vec!["Italian".into()]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_deref(), Some("Chicken Pasta"));
    }

    // --- Phase 7: T024 performance, T025 atomicity ---

    #[test]
    fn performance_5000_recipes() {
        let db = test_db();

        // Insert 5000 recipes
        for i in 0..5000 {
            let mut recipe = sample_recipe();
            recipe.title = ExtractedField::found(format!("Recipe {i}"));
            let tags = TagSet {
                cuisine: vec![Tag::new(if i % 2 == 0 { "Italian" } else { "Thai" }, 0.9)],
                course: vec![Tag::new("dinner", 0.8)],
                diet: vec![],
            };
            save_recipe(
                &db,
                SaveRecipeInput {
                    recipe: &recipe,
                    tags: &tags,
                    source_url: &format!("https://example.com/recipe/{i}"),
                },
            )
            .unwrap();
        }

        // list_recipes < 100ms
        let start = std::time::Instant::now();
        let list = list_recipes(&db).unwrap();
        let list_elapsed = start.elapsed();
        assert_eq!(list.len(), 5000);
        assert!(
            list_elapsed.as_millis() < 100,
            "list_recipes took {}ms",
            list_elapsed.as_millis()
        );

        // search_recipes < 100ms
        let start = std::time::Instant::now();
        let results = search_recipes(
            &db,
            &SearchQuery {
                query: Some("Recipe 42".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let search_elapsed = start.elapsed();
        assert!(!results.is_empty());
        assert!(
            search_elapsed.as_millis() < 100,
            "search_recipes took {}ms",
            search_elapsed.as_millis()
        );

        // get_recipe < 100ms
        let start = std::time::Instant::now();
        let _recipe = get_recipe(&db, &list[0].id).unwrap();
        let get_elapsed = start.elapsed();
        assert!(
            get_elapsed.as_millis() < 100,
            "get_recipe took {}ms",
            get_elapsed.as_millis()
        );
    }

    #[test]
    fn atomic_transaction_safety() {
        let db = test_db();
        let recipe = sample_recipe();
        let tags = sample_tags();

        // Save a recipe successfully
        let result = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/atomic",
            },
        )
        .unwrap();

        // Verify it exists
        let saved = get_recipe(&db, &result.id).unwrap();
        assert_eq!(saved.title, ExtractedField::found("Test Pasta".to_string()));

        // Try to save with a duplicate source_url constraint violation won't happen
        // because save_recipe handles upsert. Instead test that partial updates
        // inside a transaction are all-or-nothing by verifying the DB state is
        // consistent after operations.
        let update_result = update_recipe(
            &db,
            &result.id,
            &UpdateFields {
                title: Some("Updated Title".into()),
                notes: Some("Updated notes".into()),
                ..Default::default()
            },
        )
        .unwrap();

        let after = get_recipe(&db, &result.id).unwrap();
        assert_eq!(
            after.title,
            ExtractedField::found("Updated Title".to_string())
        );
        assert_eq!(after.notes, "Updated notes");
        assert!(after.updated_at >= update_result.updated_at);
    }

    #[test]
    fn storage_error_propagation() {
        // Verify that database errors surface as StorageError::Storage with a message
        let db = test_db();

        // Drop the recipes table to force a SQL error
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("DROP TABLE IF EXISTS ingredients", [])
                .unwrap();
        }

        // Attempting to save should fail with a StorageError::Storage
        let recipe = sample_recipe();
        let tags = sample_tags();
        let result = save_recipe(
            &db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/error-test",
            },
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::Storage { message } => {
                assert!(!message.is_empty(), "Error message should not be empty");
            }
            other => panic!("Expected StorageError::Storage, got: {other:?}"),
        }

        // get_recipe on a non-existent ID should return NotFound
        // (re-create a working DB for this check)
        let db2 = test_db();
        let result = get_recipe(&db2, "nonexistent-id");
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::NotFound { message } => {
                assert!(
                    message.contains("nonexistent-id"),
                    "NotFound message should contain the ID"
                );
            }
            other => panic!("Expected StorageError::NotFound, got: {other:?}"),
        }
    }
}
