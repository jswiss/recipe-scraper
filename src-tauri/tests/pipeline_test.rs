//! Integration tests for the full recipe-scraper pipeline:
//! extract -> tag -> persist -> retrieve.
//!
//! These tests exercise the public API across module boundaries, verifying
//! that JSON-LD and Microdata HTML flows through extraction, tagging, and
//! storage without data loss.

use recipe_scraper_lib::recipe_extraction::{
    extract_recipe, ExtractedField, ExtractionError, ExtractionSource,
};
use recipe_scraper_lib::recipe_tagging::tag_recipe_from_extracted;
use recipe_scraper_lib::storage::repository;
use recipe_scraper_lib::storage::{Database, SaveRecipeInput};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const JSONLD_HTML: &str = include_str!("fixtures/jsonld_recipe.html");
const MICRODATA_HTML: &str = include_str!("fixtures/microdata_recipe.html");
const NO_RECIPE_HTML: &str = include_str!("fixtures/no_recipe.html");
const MALFORMED_JSONLD_HTML: &str = include_str!("fixtures/malformed_jsonld.html");

// ---------------------------------------------------------------------------
// T008 — JSON-LD extract -> tag -> persist round-trip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_jsonld_extract_tag_persist_roundtrip() {
    // --- Extract ---
    let recipe = extract_recipe(JSONLD_HTML.to_string())
        .await
        .expect("JSON-LD extraction should succeed");

    assert_eq!(
        recipe.title.value(),
        Some(&"Classic Chocolate Chip Cookies".to_string()),
        "Title should match fixture"
    );
    assert_eq!(recipe.ingredients.len(), 6, "Should have 6 ingredients");
    assert_eq!(recipe.instructions.len(), 4, "Should have 4 instructions");
    assert_eq!(recipe.source, ExtractionSource::JsonLd);

    // Verify time fields
    assert_eq!(
        recipe.prep_time_minutes,
        ExtractedField::found(15),
        "Prep time should be 15 minutes"
    );
    assert_eq!(
        recipe.cook_time_minutes,
        ExtractedField::found(12),
        "Cook time should be 12 minutes"
    );

    // Verify servings
    assert!(recipe.servings.is_found(), "Servings should be found");
    assert!(
        recipe.servings.value().unwrap().contains("24"),
        "Servings should mention 24"
    );

    // Verify images
    assert!(recipe.images.is_found(), "Images should be found");
    let images = recipe.images.value().unwrap();
    assert!(!images.is_empty(), "Should have at least 1 image URL");

    // Verify nutrition
    assert!(recipe.nutrition.is_found(), "Nutrition should be found");
    let nutrition = recipe.nutrition.value().unwrap();
    assert_eq!(nutrition.calories, Some(220), "Calories should be 220");
    assert_eq!(nutrition.fat_grams, Some(11.0), "Fat should be 11g");
    assert_eq!(nutrition.carbs_grams, Some(28.0), "Carbs should be 28g");
    assert_eq!(nutrition.protein_grams, Some(3.0), "Protein should be 3g");

    // --- Tag ---
    let tags = tag_recipe_from_extracted(&recipe, false);

    let course_labels: Vec<&str> = tags.course.iter().map(|t| t.label.as_str()).collect();
    assert!(
        course_labels.iter().any(|l| l.to_lowercase() == "dessert"),
        "Course tags should include 'dessert', got: {:?}",
        course_labels
    );

    // --- Persist and retrieve ---
    let db = Database::new_in_memory().expect("In-memory DB should initialize");

    let save_result = repository::save_recipe(
        &db,
        SaveRecipeInput {
            recipe: &recipe,
            tags: &tags,
            source_url: "https://example.com/cookies",
        },
    )
    .expect("save_recipe should succeed");
    assert!(save_result.created, "First save should create a new record");

    let saved = repository::get_recipe(&db, &save_result.id)
        .expect("get_recipe should find the saved recipe");

    // Verify round-trip fidelity
    assert_eq!(saved.source_url, "https://example.com/cookies");
    assert_eq!(
        saved.title,
        ExtractedField::found("Classic Chocolate Chip Cookies".to_string())
    );
    assert_eq!(saved.ingredients.len(), 6);
    assert_eq!(saved.instructions.len(), 4);
    assert_eq!(saved.prep_time_minutes, ExtractedField::found(15));
    assert_eq!(saved.cook_time_minutes, ExtractedField::found(12));
    assert_eq!(saved.extraction_source, ExtractionSource::JsonLd);

    // Verify tags survived the round-trip
    let saved_course_labels: Vec<&str> =
        saved.tags.course.iter().map(|t| t.label.as_str()).collect();
    assert!(
        saved_course_labels
            .iter()
            .any(|l| l.to_lowercase() == "dessert"),
        "Persisted course tags should include 'dessert', got: {:?}",
        saved_course_labels
    );
}

// ---------------------------------------------------------------------------
// T009 — Microdata extract -> tag -> persist round-trip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_microdata_extract_tag_persist_roundtrip() {
    // --- Extract ---
    let recipe = extract_recipe(MICRODATA_HTML.to_string())
        .await
        .expect("Microdata extraction should succeed");

    assert_eq!(
        recipe.title.value(),
        Some(&"Classic Chocolate Chip Cookies".to_string()),
        "Title should match fixture"
    );
    assert_eq!(recipe.ingredients.len(), 6, "Should have 6 ingredients");
    assert_eq!(recipe.instructions.len(), 4, "Should have 4 instructions");
    assert_eq!(recipe.source, ExtractionSource::Microdata);

    // Verify time fields
    assert_eq!(
        recipe.prep_time_minutes,
        ExtractedField::found(15),
        "Prep time should be 15 minutes"
    );
    assert_eq!(
        recipe.cook_time_minutes,
        ExtractedField::found(12),
        "Cook time should be 12 minutes"
    );

    // Verify servings
    assert!(recipe.servings.is_found(), "Servings should be found");

    // --- Tag ---
    let tags = tag_recipe_from_extracted(&recipe, false);

    let course_labels: Vec<&str> = tags.course.iter().map(|t| t.label.as_str()).collect();
    assert!(
        course_labels.iter().any(|l| l.to_lowercase() == "dessert"),
        "Course tags should include 'dessert', got: {:?}",
        course_labels
    );

    // --- Persist and retrieve ---
    let db = Database::new_in_memory().expect("In-memory DB should initialize");

    let save_result = repository::save_recipe(
        &db,
        SaveRecipeInput {
            recipe: &recipe,
            tags: &tags,
            source_url: "https://example.com/cookies-microdata",
        },
    )
    .expect("save_recipe should succeed");
    assert!(save_result.created, "First save should create a new record");

    let saved = repository::get_recipe(&db, &save_result.id)
        .expect("get_recipe should find the saved recipe");

    // Verify round-trip fidelity
    assert_eq!(saved.source_url, "https://example.com/cookies-microdata");
    assert_eq!(
        saved.title,
        ExtractedField::found("Classic Chocolate Chip Cookies".to_string())
    );
    assert_eq!(saved.ingredients.len(), 6);
    assert_eq!(saved.instructions.len(), 4);
    assert_eq!(saved.prep_time_minutes, ExtractedField::found(15));
    assert_eq!(saved.cook_time_minutes, ExtractedField::found(12));
    assert_eq!(saved.extraction_source, ExtractionSource::Microdata);

    // Verify tags survived the round-trip
    let saved_course_labels: Vec<&str> =
        saved.tags.course.iter().map(|t| t.label.as_str()).collect();
    assert!(
        saved_course_labels
            .iter()
            .any(|l| l.to_lowercase() == "dessert"),
        "Persisted course tags should include 'dessert', got: {:?}",
        saved_course_labels
    );
}

// ---------------------------------------------------------------------------
// T010 — No-recipe HTML returns ExtractionError::NoRecipeFound
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_recipe_html_returns_extraction_error() {
    let result = extract_recipe(NO_RECIPE_HTML.to_string()).await;

    assert!(
        result.is_err(),
        "Extraction should fail for non-recipe HTML"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, ExtractionError::NoRecipeFound { .. }),
        "Error should be NoRecipeFound, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// T011 — Malformed JSON-LD returns an error (not a panic)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_malformed_jsonld_returns_error() {
    let result = extract_recipe(MALFORMED_JSONLD_HTML.to_string()).await;

    // The malformed JSON-LD fixture contains invalid JSON (`"broken": }`).
    // The extractor should return an error rather than panicking. It may fall
    // through to microdata (which also won't find anything), ultimately
    // producing some variant of ExtractionError.
    assert!(
        result.is_err(),
        "Extraction should fail for malformed JSON-LD without a valid microdata fallback"
    );
}
