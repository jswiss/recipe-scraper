//! Tauri commands for recipe extraction.
//!
//! Provides the IPC interface for extracting recipe data from HTML content.

use super::json_ld::extract_from_jsonld;
use super::microdata::extract_from_microdata;
use super::models::{ExtractedRecipe, ExtractionError, ExtractionResult};

/// Extracts recipe data from HTML content.
///
/// Uses a priority chain:
/// 1. JSON-LD structured data (most reliable)
/// 2. Microdata/itemscope markup (fallback for older sites)
///
/// Returns an ExtractedRecipe on success, or an ExtractionError if no recipe
/// could be found or extracted.
#[tauri::command]
pub async fn extract_recipe(html: String) -> Result<ExtractedRecipe, ExtractionError> {
    extract_recipe_internal(&html)
}

/// Internal extraction function that can be called synchronously.
fn extract_recipe_internal(html: &str) -> ExtractionResult {
    // Try JSON-LD first (priority 1)
    match extract_from_jsonld(html) {
        Ok(recipe) => {
            log::info!(
                "Successfully extracted recipe from JSON-LD: {:?}",
                recipe.title.value()
            );
            return Ok(recipe);
        }
        Err(e) => {
            log::debug!("JSON-LD extraction failed: {}", e);
        }
    }

    // Try Microdata (priority 2)
    match extract_from_microdata(html) {
        Ok(recipe) => {
            log::info!(
                "Successfully extracted recipe from Microdata: {:?}",
                recipe.title.value()
            );
            return Ok(recipe);
        }
        Err(e) => {
            log::debug!("Microdata extraction failed: {}", e);
        }
    }

    // No structured data found
    Err(ExtractionError::no_recipe_found(
        "No structured recipe data found (JSON-LD or Microdata)",
        html,
    ))
}

/// Detects if HTML content likely contains a recipe.
///
/// Returns true if the HTML contains recipe-related structured data markers.
#[allow(dead_code)]
pub fn detect_recipe_content(html: &str) -> bool {
    let html_lower = html.to_lowercase();

    // Check for JSON-LD recipe markers
    if html_lower.contains("application/ld+json") && html_lower.contains("recipe") {
        return true;
    }

    // Check for Microdata recipe markers
    if html_lower.contains("itemtype") && html_lower.contains("recipe") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::models::ExtractionSource;

    #[test]
    fn test_extract_recipe_jsonld() {
        let html = r#"
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Recipe",
                    "name": "Test Recipe",
                    "recipeIngredient": ["flour", "sugar"],
                    "recipeInstructions": ["Mix", "Bake"]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = extract_recipe_internal(html);
        assert!(result.is_ok());
        let recipe = result.unwrap();
        assert_eq!(recipe.source, ExtractionSource::JsonLd);
    }

    #[test]
    fn test_extract_recipe_microdata() {
        let html = r#"
            <html>
            <body>
                <div itemscope itemtype="https://schema.org/Recipe">
                    <h1 itemprop="name">Microdata Recipe</h1>
                    <li itemprop="recipeIngredient">flour</li>
                </div>
            </body>
            </html>
        "#;

        let result = extract_recipe_internal(html);
        assert!(result.is_ok());
        let recipe = result.unwrap();
        assert_eq!(recipe.source, ExtractionSource::Microdata);
    }

    #[test]
    fn test_extract_recipe_prefers_jsonld() {
        // HTML with both JSON-LD and Microdata - should prefer JSON-LD
        let html = r#"
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Recipe",
                    "name": "JSON-LD Recipe",
                    "recipeIngredient": ["flour"]
                }
                </script>
            </head>
            <body>
                <div itemscope itemtype="https://schema.org/Recipe">
                    <h1 itemprop="name">Microdata Recipe</h1>
                    <li itemprop="recipeIngredient">sugar</li>
                </div>
            </body>
            </html>
        "#;

        let result = extract_recipe_internal(html);
        assert!(result.is_ok());
        let recipe = result.unwrap();
        assert_eq!(recipe.source, ExtractionSource::JsonLd);
        assert!(recipe
            .title
            .value()
            .map(|s| s.contains("JSON-LD"))
            .unwrap_or(false));
    }

    #[test]
    fn test_extract_recipe_no_content() {
        let html = "<html><body><p>No recipe here</p></body></html>";

        let result = extract_recipe_internal(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtractionError::NoRecipeFound { .. }
        ));
    }

    #[test]
    fn test_detect_recipe_content() {
        // JSON-LD recipe
        assert!(detect_recipe_content(
            r#"<script type="application/ld+json">{"@type":"Recipe"}</script>"#
        ));

        // Microdata recipe
        assert!(detect_recipe_content(
            r#"<div itemtype="https://schema.org/Recipe">"#
        ));

        // No recipe
        assert!(!detect_recipe_content("<html><body>Hello</body></html>"));
    }
}
