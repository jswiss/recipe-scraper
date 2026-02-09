use crate::recipe_extraction::ExtractedRecipe;

use super::models::Tag;
use super::scoring::finalize_tags;
use super::vocabulary::cuisine_vocabulary;

/// Assigns cuisine tags to a recipe based on title, ingredients, and instructions.
///
/// Confidence formula per research.md R2:
///   confidence = 0.35 * title_signal + 0.35 * ingredient_signal + 0.30 * instruction_signal
pub fn tag(recipe: &ExtractedRecipe) -> Vec<Tag> {
    let title_lower = recipe
        .title
        .value()
        .map(|t| t.to_lowercase())
        .unwrap_or_default();

    let ingredient_names: Vec<String> = recipe
        .ingredients
        .iter()
        .map(|i| i.name.to_lowercase())
        .collect();

    let instruction_text: String = recipe
        .instructions
        .iter()
        .map(|i| i.text.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    let mut tags = Vec::new();

    for entry in cuisine_vocabulary() {
        let title_signal = if entry
            .title_keywords
            .iter()
            .any(|kw| title_lower.contains(kw))
        {
            1.0
        } else {
            0.0
        };

        let matched_ingredients = entry
            .ingredient_keywords
            .iter()
            .filter(|kw| ingredient_names.iter().any(|name| name.contains(*kw)))
            .count();
        let ingredient_signal = (matched_ingredients as f64 / 2.0).min(1.0);

        let matched_instructions = entry
            .instruction_keywords
            .iter()
            .filter(|kw| instruction_text.contains(*kw))
            .count();
        let instruction_signal = (matched_instructions as f64 / 1.0).min(1.0);

        let confidence = 0.35 * title_signal + 0.35 * ingredient_signal + 0.30 * instruction_signal;

        if confidence > 0.0 {
            tags.push(Tag::new(entry.label, confidence));
        }
    }

    finalize_tags(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::{ExtractedField, ExtractionSource, Ingredient, Instruction};

    fn make_recipe(
        title: Option<&str>,
        ingredients: Vec<&str>,
        instructions: Vec<&str>,
    ) -> ExtractedRecipe {
        let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        if let Some(t) = title {
            recipe.title = ExtractedField::found(t.to_string());
        }
        recipe.ingredients = ingredients
            .into_iter()
            .map(|name| Ingredient::new(name, None, None, name))
            .collect();
        recipe.instructions = instructions
            .into_iter()
            .enumerate()
            .map(|(i, text)| Instruction::new(i as u32 + 1, text))
            .collect();
        recipe
    }

    #[test]
    fn test_thai_recipe_clear_indicators() {
        let recipe = make_recipe(
            Some("Pad Thai"),
            vec!["fish sauce", "rice noodles", "tamarind", "thai basil"],
            vec!["stir-fry in wok"],
        );
        let tags = tag(&recipe);
        let thai = tags.iter().find(|t| t.label == "Thai");
        assert!(thai.is_some(), "Expected Thai tag, got: {:?}", tags);
        assert!(thai.unwrap().confidence > 0.7);
    }

    #[test]
    fn test_italian_recipe_multiple_tags() {
        let recipe = make_recipe(
            Some("Pasta Primavera"),
            vec!["pasta", "olive oil", "parmesan", "basil"],
            vec!["cook al dente"],
        );
        let tags = tag(&recipe);
        let italian = tags.iter().find(|t| t.label == "Italian");
        assert!(italian.is_some(), "Expected Italian tag");
        assert_eq!(tags[0].label, "Italian", "Italian should be highest");
    }

    #[test]
    fn test_asian_cuisine_from_ingredients() {
        let recipe = make_recipe(
            Some("Stir-Fry Noodle Bowl"),
            vec![
                "soy sauce",
                "rice noodles",
                "sesame oil",
                "ginger",
                "hoisin",
            ],
            vec!["stir-fry in wok"],
        );
        let tags = tag(&recipe);
        assert!(!tags.is_empty(), "Expected some Asian cuisine tags");
    }

    #[test]
    fn test_no_recognizable_indicators() {
        let recipe = make_recipe(
            Some("My Special Dish"),
            vec!["water", "salt", "pepper"],
            vec!["mix together"],
        );
        let tags = tag(&recipe);
        assert!(
            tags.is_empty(),
            "Expected no cuisine tags for generic recipe"
        );
    }

    #[test]
    fn test_missing_title_tags_from_ingredients() {
        let recipe = make_recipe(
            None,
            vec!["tortilla", "jalapeño", "cilantro", "cumin", "salsa"],
            vec!["roast peppers"],
        );
        let tags = tag(&recipe);
        assert!(
            !tags.is_empty(),
            "Should tag from ingredients even without title"
        );
    }
}
