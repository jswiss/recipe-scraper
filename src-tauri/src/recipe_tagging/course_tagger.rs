use crate::recipe_extraction::ExtractedRecipe;

use super::models::Tag;
use super::scoring::finalize_tags;
use super::vocabulary::course_vocabulary;

/// Assigns course tags to a recipe based on title, description, ingredients, and contextual cues.
///
/// Confidence formula per research.md R2:
///   confidence = 0.40 * title_signal + 0.25 * ingredient_signal
///              + 0.20 * description_signal + 0.15 * contextual_signal
pub fn tag(recipe: &ExtractedRecipe) -> Vec<Tag> {
    let title_lower = recipe
        .title
        .value()
        .map(|t| t.to_lowercase())
        .unwrap_or_default();

    let description_lower = recipe
        .description
        .value()
        .map(|d| d.to_lowercase())
        .unwrap_or_default();

    let ingredient_names: Vec<String> = recipe
        .ingredients
        .iter()
        .map(|i| i.name.to_lowercase())
        .collect();

    let all_text = format!(
        "{} {} {}",
        title_lower,
        description_lower,
        recipe
            .instructions
            .iter()
            .map(|i| i.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut tags = Vec::new();

    for entry in course_vocabulary() {
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

        let description_signal = if entry
            .title_keywords
            .iter()
            .any(|kw| description_lower.contains(kw))
        {
            1.0
        } else {
            0.0
        };

        let matched_contextual = entry
            .contextual_keywords
            .iter()
            .filter(|kw| all_text.contains(*kw))
            .count();
        let contextual_signal = (matched_contextual as f64 / 1.0).min(1.0);

        let confidence = 0.40 * title_signal
            + 0.25 * ingredient_signal
            + 0.20 * description_signal
            + 0.15 * contextual_signal;

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
        description: Option<&str>,
        ingredients: Vec<&str>,
        instructions: Vec<&str>,
    ) -> ExtractedRecipe {
        let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        if let Some(t) = title {
            recipe.title = ExtractedField::found(t.to_string());
        }
        if let Some(d) = description {
            recipe.description = ExtractedField::found(d.to_string());
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
    fn test_blueberry_pancakes_breakfast() {
        let recipe = make_recipe(
            Some("Blueberry Breakfast Pancakes"),
            Some("A perfect breakfast recipe"),
            vec!["flour", "eggs", "blueberries", "maple syrup", "bacon"],
            vec!["cook on griddle in the morning"],
        );
        let tags = tag(&recipe);
        let breakfast = tags.iter().find(|t| t.label == "breakfast");
        assert!(breakfast.is_some(), "Expected breakfast tag, got: {:?}", tags);
        assert!(breakfast.unwrap().confidence > 0.7);
    }

    #[test]
    fn test_chocolate_cake_dessert() {
        let recipe = make_recipe(
            Some("Chocolate Cake"),
            None,
            vec!["sugar", "cocoa", "vanilla extract", "frosting"],
            vec!["bake at 350", "apply frosting"],
        );
        let tags = tag(&recipe);
        assert!(!tags.is_empty());
        assert_eq!(tags[0].label, "dessert", "Dessert should be highest");
    }

    #[test]
    fn test_salad_multiple_courses() {
        let recipe = make_recipe(
            Some("Caesar Salad"),
            None,
            vec!["lettuce", "croutons", "parmesan", "dressing"],
            vec!["toss ingredients"],
        );
        let tags = tag(&recipe);
        let salad = tags.iter().find(|t| t.label == "salad");
        assert!(salad.is_some(), "Expected salad tag, got: {:?}", tags);
    }

    #[test]
    fn test_explicit_course_in_title() {
        let recipe = make_recipe(
            Some("Appetizer: Bruschetta"),
            Some("A perfect starter for any party"),
            vec!["puff pastry", "cream cheese", "olive oil"],
            vec!["serve as finger food before dinner"],
        );
        let tags = tag(&recipe);
        let appetizer = tags.iter().find(|t| t.label == "appetizer");
        assert!(appetizer.is_some(), "Expected appetizer tag, got: {:?}", tags);
    }

    #[test]
    fn test_no_course_indicators() {
        let recipe = make_recipe(
            Some("Mystery Dish"),
            None,
            vec!["water", "salt"],
            vec!["combine"],
        );
        let tags = tag(&recipe);
        assert!(tags.is_empty(), "Expected no course tags");
    }
}
