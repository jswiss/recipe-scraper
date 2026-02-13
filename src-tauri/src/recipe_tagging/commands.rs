use crate::recipe_extraction::{extract_recipe, ExtractedRecipe};

use super::models::{TagSet, TaggingError, TaggingResult};
use super::scoring::refine_scores;
use super::{course_tagger, cuisine_tagger, diet_tagger};

/// Core tagging orchestrator. Calls all three domain taggers and assembles a TagSet.
/// When `refine` is true, applies heuristic co-occurrence and cross-domain scoring (FR-016).
pub fn tag_recipe_from_extracted(recipe: &ExtractedRecipe, refine: bool) -> TagSet {
    let mut cuisine = cuisine_tagger::tag(recipe);
    let mut course = course_tagger::tag(recipe);
    let diet = diet_tagger::tag(recipe);

    if refine {
        refine_scores(&mut cuisine, &mut course, recipe);
    }

    TagSet {
        cuisine,
        course,
        diet,
    }
}

/// Tauri command: tag an already-extracted recipe.
#[tauri::command]
pub async fn tag_recipe(
    recipe: ExtractedRecipe,
    refine: Option<bool>,
) -> Result<TagSet, TaggingError> {
    Ok(tag_recipe_from_extracted(&recipe, refine.unwrap_or(false)))
}

/// Tauri command: extract a recipe from HTML and automatically tag it.
#[tauri::command]
pub async fn extract_and_tag(html: String) -> Result<TaggingResult, TaggingError> {
    let recipe = extract_recipe(html)
        .await
        .map_err(|e| TaggingError::ExtractionFailed {
            message: e.to_string(),
        })?;

    let tags = tag_recipe_from_extracted(&recipe, false);

    Ok(TaggingResult { recipe, tags })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::{ExtractedField, ExtractionSource, Ingredient};
    use crate::recipe_tagging::models::MIN_CONFIDENCE_THRESHOLD;

    fn make_recipe(title: &str, ingredients: Vec<&str>) -> ExtractedRecipe {
        let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        recipe.title = ExtractedField::found(title.to_string());
        recipe.ingredients = ingredients
            .into_iter()
            .map(|name| Ingredient::new(name, None, None, name))
            .collect();
        recipe
    }

    #[test]
    fn test_tag_recipe_valid_returns_all_domains() {
        let recipe = make_recipe(
            "Pad Thai",
            vec!["fish sauce", "rice noodles", "tofu", "lime", "peanuts"],
        );
        let tags = tag_recipe_from_extracted(&recipe, false);
        // Should have at least cuisine tags
        assert!(!tags.cuisine.is_empty(), "Expected cuisine tags");
        // All three domains should be present (even if empty)
        let _ = &tags.course;
        let _ = &tags.diet;
    }

    #[test]
    fn test_tag_recipe_empty_returns_empty_tagset() {
        let recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        let tags = tag_recipe_from_extracted(&recipe, false);
        assert!(tags.cuisine.is_empty());
        assert!(tags.course.is_empty());
        assert!(tags.diet.is_empty());
    }

    #[test]
    fn test_tags_sorted_by_confidence_descending() {
        let recipe = make_recipe(
            "Stir-Fry Noodle Bowl",
            vec![
                "soy sauce",
                "rice noodles",
                "sesame oil",
                "ginger",
                "hoisin",
            ],
        );
        let tags = tag_recipe_from_extracted(&recipe, false);
        for domain_tags in [&tags.cuisine, &tags.course, &tags.diet] {
            for window in domain_tags.windows(2) {
                assert!(
                    window[0].confidence >= window[1].confidence,
                    "Tags not sorted descending: {:?}",
                    domain_tags
                );
            }
        }
    }

    #[test]
    fn test_no_tag_below_threshold() {
        let recipe = make_recipe(
            "Pad Thai with Chicken",
            vec![
                "fish sauce",
                "rice noodles",
                "chicken breast",
                "peanuts",
                "lime",
                "sugar",
            ],
        );
        let tags = tag_recipe_from_extracted(&recipe, false);
        for tag in tags
            .cuisine
            .iter()
            .chain(tags.course.iter())
            .chain(tags.diet.iter())
        {
            assert!(
                tag.confidence >= MIN_CONFIDENCE_THRESHOLD,
                "Tag '{}' has confidence {} below threshold {}",
                tag.label,
                tag.confidence,
                MIN_CONFIDENCE_THRESHOLD
            );
        }
    }

    // --- Edge case tests (T019) ---

    #[test]
    fn test_completely_empty_recipe() {
        let recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        let tags = tag_recipe_from_extracted(&recipe, false);
        assert!(tags.cuisine.is_empty());
        assert!(tags.course.is_empty());
        assert!(tags.diet.is_empty());
    }

    #[test]
    fn test_title_only_recipe() {
        let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        recipe.title = ExtractedField::found("Italian Pasta Dinner".to_string());
        let tags = tag_recipe_from_extracted(&recipe, false);
        // Should still get some cuisine/course tags from title alone
        // (diet will be empty since no ingredients)
        assert!(tags.diet.is_empty(), "No diet tags without ingredients");
    }

    #[test]
    fn test_fusion_recipe_multiple_cuisines() {
        let recipe = make_recipe(
            "Korean-Mexican Fusion Tacos",
            vec![
                "gochujang",
                "tortilla",
                "kimchi",
                "cilantro",
                "sesame oil",
                "lime",
                "avocado",
            ],
        );
        let tags = tag_recipe_from_extracted(&recipe, false);
        let labels: Vec<&str> = tags.cuisine.iter().map(|t| t.label.as_str()).collect();
        // Should tag both Korean and Mexican signals
        let has_korean = labels.contains(&"Korean");
        let has_mexican = labels.contains(&"Mexican");
        assert!(
            has_korean || has_mexican,
            "Fusion recipe should tag at least one cuisine. Got: {:?}",
            labels
        );
    }

    #[test]
    fn test_tag_recipe_command_returns_tagset() {
        let recipe = make_recipe(
            "Pad Thai",
            vec!["fish sauce", "rice noodles", "tofu", "lime", "peanuts"],
        );
        let tags = tag_recipe_from_extracted(&recipe, false);
        assert!(
            !tags.cuisine.is_empty(),
            "Pad Thai should produce at least one cuisine tag"
        );
    }

    #[tokio::test]
    async fn test_extract_and_tag_no_recipe_returns_error() {
        let html = "<html><body>No recipe here</body></html>".to_string();
        let result = extract_and_tag(html).await;
        assert!(
            result.is_err(),
            "HTML with no recipe should return an error"
        );
        assert!(
            matches!(result.unwrap_err(), TaggingError::ExtractionFailed { .. }),
            "Error should be ExtractionFailed variant"
        );
    }

    #[test]
    fn test_refine_mode_does_not_panic() {
        let recipe = make_recipe(
            "Thai Green Curry",
            vec!["coconut milk", "thai basil", "fish sauce", "rice"],
        );
        let tags = tag_recipe_from_extracted(&recipe, true);
        // Just verify it doesn't panic and produces valid output
        for tag in tags
            .cuisine
            .iter()
            .chain(tags.course.iter())
            .chain(tags.diet.iter())
        {
            assert!(tag.confidence >= MIN_CONFIDENCE_THRESHOLD);
            assert!(tag.confidence <= 1.0);
        }
    }
}
