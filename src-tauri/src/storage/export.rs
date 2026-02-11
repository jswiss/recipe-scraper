use serde::{Deserialize, Serialize};

use crate::recipe_extraction::{
    ExtractedField, ExtractionSource, Ingredient, Instruction, NutritionInfo,
};
use crate::recipe_tagging::{Tag, TagSet};

use super::database::Database;
use super::models::*;
use super::repository;

// --- T013: Schema.org serialization structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOrgRecipe {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub type_field: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "recipeIngredient")]
    pub recipe_ingredient: Vec<String>,
    #[serde(rename = "recipeInstructions")]
    pub recipe_instructions: Vec<SchemaOrgHowToStep>,
    #[serde(rename = "prepTime", skip_serializing_if = "Option::is_none")]
    pub prep_time: Option<String>,
    #[serde(rename = "cookTime", skip_serializing_if = "Option::is_none")]
    pub cook_time: Option<String>,
    #[serde(rename = "recipeYield", skip_serializing_if = "Option::is_none")]
    pub recipe_yield: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nutrition: Option<SchemaOrgNutrition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    // Extension fields (not standard schema.org but preserved for round-trip)
    #[serde(rename = "x-extractionSource", skip_serializing_if = "Option::is_none")]
    pub extraction_source: Option<String>,
    #[serde(rename = "x-tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<SchemaOrgTags>,
    #[serde(rename = "x-notes", skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(
        rename = "x-ingredientsParsed",
        skip_serializing_if = "Option::is_none"
    )]
    pub ingredients_parsed: Option<Vec<SchemaOrgIngredient>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOrgHowToStep {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub text: String,
    pub position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOrgNutrition {
    #[serde(rename = "@type")]
    pub type_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<String>,
    #[serde(rename = "fatContent", skip_serializing_if = "Option::is_none")]
    pub fat_content: Option<String>,
    #[serde(
        rename = "saturatedFatContent",
        skip_serializing_if = "Option::is_none"
    )]
    pub saturated_fat_content: Option<String>,
    #[serde(
        rename = "carbohydrateContent",
        skip_serializing_if = "Option::is_none"
    )]
    pub carbohydrate_content: Option<String>,
    #[serde(rename = "fiberContent", skip_serializing_if = "Option::is_none")]
    pub fiber_content: Option<String>,
    #[serde(rename = "sugarContent", skip_serializing_if = "Option::is_none")]
    pub sugar_content: Option<String>,
    #[serde(rename = "proteinContent", skip_serializing_if = "Option::is_none")]
    pub protein_content: Option<String>,
    #[serde(rename = "sodiumContent", skip_serializing_if = "Option::is_none")]
    pub sodium_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOrgTagEntry {
    pub label: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOrgTags {
    pub cuisine: Vec<SchemaOrgTagEntry>,
    pub course: Vec<SchemaOrgTagEntry>,
    pub diet: Vec<SchemaOrgTagEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOrgIngredient {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

// --- Bidirectional conversion ---

pub fn saved_recipe_to_schema_org(recipe: &SavedRecipe) -> SchemaOrgRecipe {
    let instructions = recipe
        .instructions
        .iter()
        .map(|i| SchemaOrgHowToStep {
            type_field: "HowToStep".into(),
            text: i.text.clone(),
            position: i.step_number,
        })
        .collect();

    let nutrition = match &recipe.nutrition {
        ExtractedField::Found { value } => Some(nutrition_to_schema_org(value)),
        _ => None,
    };

    let source_str = match recipe.extraction_source {
        ExtractionSource::JsonLd => "json_ld",
        ExtractionSource::Microdata => "microdata",
        ExtractionSource::AiFallback => "ai_fallback",
    };

    SchemaOrgRecipe {
        context: "https://schema.org".into(),
        type_field: "Recipe".into(),
        name: recipe.title.value().cloned(),
        description: recipe.description.value().cloned(),
        recipe_ingredient: recipe
            .ingredients
            .iter()
            .map(|i| i.raw_text.clone())
            .collect(),
        recipe_instructions: instructions,
        prep_time: recipe.prep_time_minutes.value().map(|m| format!("PT{m}M")),
        cook_time: recipe.cook_time_minutes.value().map(|m| format!("PT{m}M")),
        recipe_yield: recipe.servings.value().cloned(),
        image: recipe.images.value().cloned(),
        nutrition,
        url: Some(recipe.source_url.clone()),
        extraction_source: Some(source_str.into()),
        tags: Some(SchemaOrgTags {
            cuisine: recipe
                .tags
                .cuisine
                .iter()
                .map(|t| SchemaOrgTagEntry {
                    label: t.label.clone(),
                    confidence: t.confidence,
                })
                .collect(),
            course: recipe
                .tags
                .course
                .iter()
                .map(|t| SchemaOrgTagEntry {
                    label: t.label.clone(),
                    confidence: t.confidence,
                })
                .collect(),
            diet: recipe
                .tags
                .diet
                .iter()
                .map(|t| SchemaOrgTagEntry {
                    label: t.label.clone(),
                    confidence: t.confidence,
                })
                .collect(),
        }),
        notes: if recipe.notes.is_empty() {
            None
        } else {
            Some(recipe.notes.clone())
        },
        ingredients_parsed: Some(
            recipe
                .ingredients
                .iter()
                .map(|i| SchemaOrgIngredient {
                    name: i.name.clone(),
                    quantity: i.quantity,
                    unit: i.unit.clone(),
                    raw_text: i.raw_text.clone(),
                })
                .collect(),
        ),
    }
}

fn nutrition_to_schema_org(n: &NutritionInfo) -> SchemaOrgNutrition {
    SchemaOrgNutrition {
        type_field: "NutritionInformation".into(),
        calories: n.calories.map(|c| format!("{c} calories")),
        fat_content: n.fat_grams.map(|g| format!("{g} g")),
        saturated_fat_content: n.saturated_fat_grams.map(|g| format!("{g} g")),
        carbohydrate_content: n.carbs_grams.map(|g| format!("{g} g")),
        fiber_content: n.fiber_grams.map(|g| format!("{g} g")),
        sugar_content: n.sugar_grams.map(|g| format!("{g} g")),
        protein_content: n.protein_grams.map(|g| format!("{g} g")),
        sodium_content: n.sodium_mg.map(|mg| format!("{mg} mg")),
    }
}

pub fn schema_org_to_saved_recipe_input(
    schema: &SchemaOrgRecipe,
) -> (crate::recipe_extraction::ExtractedRecipe, TagSet, String) {
    let title = match &schema.name {
        Some(n) => ExtractedField::found(n.clone()),
        None => ExtractedField::not_found("Not found in import"),
    };
    let description = match &schema.description {
        Some(d) => ExtractedField::found(d.clone()),
        None => ExtractedField::not_found("Not found in import"),
    };
    let ingredients: Vec<Ingredient> = if let Some(parsed) = &schema.ingredients_parsed {
        parsed
            .iter()
            .map(|p| {
                Ingredient::new(
                    p.name.clone(),
                    p.quantity,
                    p.unit.clone(),
                    p.raw_text.clone(),
                )
            })
            .collect()
    } else {
        schema
            .recipe_ingredient
            .iter()
            .map(|raw| Ingredient::from_raw(raw.clone()))
            .collect()
    };
    let instructions: Vec<Instruction> = schema
        .recipe_instructions
        .iter()
        .map(|s| Instruction::new(s.position, s.text.clone()))
        .collect();
    let prep_time = parse_iso_duration_minutes(schema.prep_time.as_deref());
    let cook_time = parse_iso_duration_minutes(schema.cook_time.as_deref());
    let servings = match &schema.recipe_yield {
        Some(y) => ExtractedField::found(y.clone()),
        None => ExtractedField::not_found("Not found in import"),
    };
    let images = match &schema.image {
        Some(imgs) if !imgs.is_empty() => ExtractedField::found(imgs.clone()),
        _ => ExtractedField::not_found("Not found in import"),
    };
    let nutrition = match &schema.nutrition {
        Some(n) => ExtractedField::found(schema_org_nutrition_to_info(n)),
        None => ExtractedField::not_found("Not found in import"),
    };

    let source = match schema.extraction_source.as_deref() {
        Some("microdata") => ExtractionSource::Microdata,
        Some("ai_fallback") => ExtractionSource::AiFallback,
        _ => ExtractionSource::JsonLd,
    };

    let tags = match &schema.tags {
        Some(t) => TagSet {
            cuisine: t
                .cuisine
                .iter()
                .map(|e| Tag::new(e.label.clone(), e.confidence))
                .collect(),
            course: t
                .course
                .iter()
                .map(|e| Tag::new(e.label.clone(), e.confidence))
                .collect(),
            diet: t
                .diet
                .iter()
                .map(|e| Tag::new(e.label.clone(), e.confidence))
                .collect(),
        },
        None => TagSet::empty(),
    };

    // Source URL: use provided URL or generate synthetic from title
    let source_url = schema.url.clone().unwrap_or_else(|| {
        let title_str = schema.name.as_deref().unwrap_or("unknown");
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            title_str.hash(&mut h);
            h.finish()
        };
        format!("import://title-hash/{hash:016x}")
    });

    let recipe = crate::recipe_extraction::ExtractedRecipe {
        title,
        description,
        ingredients,
        instructions,
        prep_time_minutes: prep_time,
        cook_time_minutes: cook_time,
        servings,
        images,
        nutrition,
        source,
    };

    (recipe, tags, source_url)
}

fn parse_iso_duration_minutes(s: Option<&str>) -> ExtractedField<u32> {
    match s {
        Some(dur) => {
            // Parse PT{n}M, PT{h}H, or PT{h}H{m}M
            let dur = dur.trim_start_matches("PT");
            let mut total: u32 = 0;
            let mut found = false;

            // Extract hours if present (e.g. "1H30M" or "2H")
            if let Some(h_pos) = dur.find('H') {
                if let Ok(hours) = dur[..h_pos].parse::<u32>() {
                    total += hours * 60;
                    found = true;
                }
                // Parse remaining minutes after 'H' (e.g. "30M" from "1H30M")
                let rest = &dur[h_pos + 1..];
                if let Some(m) = rest.strip_suffix('M') {
                    if let Ok(mins) = m.parse::<u32>() {
                        total += mins;
                        found = true;
                    }
                }
            } else if let Some(m) = dur.strip_suffix('M') {
                // Minutes only (e.g. "30M")
                if let Ok(mins) = m.parse::<u32>() {
                    total += mins;
                    found = true;
                }
            }

            if found {
                ExtractedField::found(total)
            } else {
                ExtractedField::not_found("Could not parse duration")
            }
        }
        None => ExtractedField::not_found("Not found in import"),
    }
}

fn schema_org_nutrition_to_info(n: &SchemaOrgNutrition) -> NutritionInfo {
    NutritionInfo {
        calories: n.calories.as_ref().and_then(|s| parse_number(s)),
        fat_grams: n.fat_content.as_ref().and_then(|s| parse_float(s)),
        saturated_fat_grams: n
            .saturated_fat_content
            .as_ref()
            .and_then(|s| parse_float(s)),
        carbs_grams: n.carbohydrate_content.as_ref().and_then(|s| parse_float(s)),
        fiber_grams: n.fiber_content.as_ref().and_then(|s| parse_float(s)),
        sugar_grams: n.sugar_content.as_ref().and_then(|s| parse_float(s)),
        protein_grams: n.protein_content.as_ref().and_then(|s| parse_float(s)),
        sodium_mg: n.sodium_content.as_ref().and_then(|s| parse_number(s)),
    }
}

fn parse_number<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.split_whitespace().next().and_then(|n| n.parse().ok())
}

fn parse_float(s: &str) -> Option<f64> {
    parse_number(s)
}

// --- T014: Export/Import file operations ---

pub fn export_recipes_to_file(
    db: &Database,
    recipe_ids: Option<&[String]>,
    file_path: &str,
) -> Result<ExportResult, StorageError> {
    let recipes = match recipe_ids {
        Some(ids) => {
            let mut result = Vec::new();
            for id in ids {
                result.push(repository::get_recipe(db, id)?);
            }
            result
        }
        None => {
            let summaries = repository::list_recipes(db)?;
            let mut result = Vec::new();
            for s in &summaries {
                result.push(repository::get_recipe(db, &s.id)?);
            }
            result
        }
    };

    let schema_recipes: Vec<SchemaOrgRecipe> =
        recipes.iter().map(saved_recipe_to_schema_org).collect();

    let json =
        serde_json::to_string_pretty(&schema_recipes).map_err(|e| StorageError::Storage {
            message: format!("Failed to serialize: {e}"),
        })?;

    std::fs::write(file_path, json).map_err(|e| StorageError::Storage {
        message: format!("Failed to write file: {e}"),
    })?;

    Ok(ExportResult {
        count: recipes.len() as i64,
        file_path: file_path.to_string(),
    })
}

pub fn import_recipes_from_file(
    db: &Database,
    file_path: &str,
) -> Result<ImportResult, StorageError> {
    let content = std::fs::read_to_string(file_path).map_err(|e| StorageError::Storage {
        message: format!("Failed to read file: {e}"),
    })?;

    let schemas: Vec<serde_json::Value> =
        serde_json::from_str(&content).map_err(|e| StorageError::Storage {
            message: format!("Failed to parse JSON: {e}"),
        })?;

    let mut imported = 0i64;
    let mut updated = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (i, value) in schemas.iter().enumerate() {
        let schema: SchemaOrgRecipe = match serde_json::from_value(value.clone()) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("Item {i}: {e}"));
                skipped += 1;
                continue;
            }
        };

        let (recipe, tags, source_url) = schema_org_to_saved_recipe_input(&schema);
        let input = SaveRecipeInput {
            recipe: &recipe,
            tags: &tags,
            source_url: &source_url,
        };

        match repository::save_recipe(db, input) {
            Ok(result) => {
                if result.created {
                    imported += 1;
                } else {
                    updated += 1;
                }
            }
            Err(e) => {
                errors.push(format!("Item {i}: {e}"));
                skipped += 1;
            }
        }
    }

    Ok(ImportResult {
        imported,
        updated,
        skipped,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::ExtractedRecipe;

    fn test_db() -> Database {
        Database::new_in_memory().expect("Failed to create test DB")
    }

    fn save_sample(db: &Database) -> String {
        let recipe = ExtractedRecipe {
            title: ExtractedField::found("Test Pasta".into()),
            description: ExtractedField::found("Delicious".into()),
            ingredients: vec![Ingredient::new(
                "pasta",
                Some(200.0),
                Some("g".into()),
                "200g pasta",
            )],
            instructions: vec![Instruction::new(1, "Cook it")],
            prep_time_minutes: ExtractedField::found(10),
            cook_time_minutes: ExtractedField::found(20),
            servings: ExtractedField::found("4".into()),
            images: ExtractedField::found(vec!["https://img.com/a.jpg".into()]),
            nutrition: ExtractedField::found(NutritionInfo {
                calories: Some(350),
                protein_grams: Some(12.0),
                ..NutritionInfo::default()
            }),
            source: ExtractionSource::JsonLd,
        };
        let tags = TagSet {
            cuisine: vec![Tag::new("Italian", 0.9)],
            course: vec![],
            diet: vec![],
        };
        repository::save_recipe(
            db,
            SaveRecipeInput {
                recipe: &recipe,
                tags: &tags,
                source_url: "https://example.com/test",
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn export_import_round_trip() {
        let db = test_db();
        let _id = save_sample(&db);

        let dir = std::env::temp_dir().join("recipe_test_export");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.json");
        let path_str = path.to_str().unwrap();

        // Export
        let export_result = export_recipes_to_file(&db, None, path_str).unwrap();
        assert_eq!(export_result.count, 1);

        // Read and verify JSON structure
        let content = std::fs::read_to_string(path_str).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed[0]["@context"], "https://schema.org");
        assert_eq!(parsed[0]["@type"], "Recipe");

        // Import into fresh DB
        let db2 = test_db();
        let import_result = import_recipes_from_file(&db2, path_str).unwrap();
        assert_eq!(import_result.imported, 1);
        assert_eq!(import_result.skipped, 0);

        // Verify fidelity
        let list = repository::list_recipes(&db2).unwrap();
        assert_eq!(list.len(), 1);
        let imported = repository::get_recipe(&db2, &list[0].id).unwrap();
        assert_eq!(imported.title, ExtractedField::found("Test Pasta".into()));
        assert_eq!(imported.ingredients.len(), 1);

        // Re-import should update, not duplicate
        let import2 = import_recipes_from_file(&db2, path_str).unwrap();
        assert_eq!(import2.updated, 1);
        assert_eq!(import2.imported, 0);
        let list2 = repository::list_recipes(&db2).unwrap();
        assert_eq!(list2.len(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn import_malformed_entries_skipped() {
        let db = test_db();
        let dir = std::env::temp_dir().join("recipe_test_malformed");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.json");
        let path_str = path.to_str().unwrap();

        // Write an array with one valid and one malformed entry
        let content = r#"[
            {"@context":"https://schema.org","@type":"Recipe","name":"Good Recipe","recipeIngredient":[],"recipeInstructions":[]},
            {"not_a_recipe": true}
        ]"#;
        std::fs::write(path_str, content).unwrap();

        let result = import_recipes_from_file(&db, path_str).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 1);
        assert!(!result.errors.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parse_iso_duration_handles_hours_and_minutes() {
        use super::parse_iso_duration_minutes;

        // Minutes only
        assert_eq!(
            parse_iso_duration_minutes(Some("PT30M")),
            ExtractedField::found(30)
        );

        // Hours and minutes
        assert_eq!(
            parse_iso_duration_minutes(Some("PT1H30M")),
            ExtractedField::found(90)
        );

        // Hours only
        assert_eq!(
            parse_iso_duration_minutes(Some("PT2H")),
            ExtractedField::found(120)
        );

        // Zero
        assert_eq!(
            parse_iso_duration_minutes(Some("PT0M")),
            ExtractedField::found(0)
        );

        // None
        assert!(matches!(
            parse_iso_duration_minutes(None),
            ExtractedField::NotFound { .. }
        ));

        // Invalid
        assert!(matches!(
            parse_iso_duration_minutes(Some("bogus")),
            ExtractedField::NotFound { .. }
        ));
    }
}
