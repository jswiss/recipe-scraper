use crate::recipe_extraction::ExtractedRecipe;

use super::models::Tag;
use super::scoring::finalize_tags;
use super::vocabulary::{diet_vocabulary, ingredient_aliases, ingredient_properties};

/// Common prefixes to strip during ingredient normalization.
const STRIP_PREFIXES: &[&str] = &[
    "organic ",
    "fresh ",
    "dried ",
    "ground ",
    "chopped ",
    "minced ",
    "diced ",
    "sliced ",
    "shredded ",
    "frozen ",
    "canned ",
    "raw ",
    "cooked ",
    "boneless ",
    "skinless ",
    "boneless skinless ",
    "extra-virgin ",
    "pure ",
    "unsweetened ",
    "roasted ",
    "toasted ",
    "crushed ",
    "grated ",
    "melted ",
    "softened ",
];

/// Normalizes an ingredient name: lowercases, strips common prefixes,
/// then looks up in the alias map.
fn normalize_ingredient(name: &str) -> String {
    let mut normalized = name.to_lowercase().trim().to_string();

    for prefix in STRIP_PREFIXES {
        if let Some(stripped) = normalized.strip_prefix(prefix) {
            normalized = stripped.to_string();
            break; // Only strip one prefix
        }
    }

    let aliases = ingredient_aliases();
    if let Some(canonical) = aliases.get(normalized.as_str()) {
        canonical.to_string()
    } else {
        normalized
    }
}

/// Assigns diet tags to a recipe using binary exclusion model per research.md R2.
///
/// For each diet: start at confidence 1.0, check each ingredient against
/// dietary flags. If violated -> 0.0. If ambiguous -> reduce by 0.2.
/// If no ingredients available -> 0.0 for all (fail-safe per SC-006).
pub fn tag(recipe: &ExtractedRecipe) -> Vec<Tag> {
    // Fail-safe: no ingredients means no diet tags
    if recipe.ingredients.is_empty() {
        return Vec::new();
    }

    let properties = ingredient_properties();
    let diet_vocab = diet_vocabulary();

    // Pre-normalize all ingredients and look up their flags
    let ingredient_flags: Vec<(String, Option<&[super::models::DietaryFlag]>)> = recipe
        .ingredients
        .iter()
        .map(|i| {
            let canonical = normalize_ingredient(&i.name);
            let flags = properties.get(canonical.as_str()).copied();
            (canonical, flags)
        })
        .collect();

    let mut tags = Vec::new();

    for diet in diet_vocab {
        let mut confidence = 1.0_f64;

        for (_canonical, flags) in &ingredient_flags {
            match flags {
                Some(ingredient_flags) => {
                    // Check if any of the ingredient's flags violate this diet
                    let violated = diet
                        .excluded_flags
                        .iter()
                        .any(|excluded| ingredient_flags.contains(excluded));
                    if violated {
                        confidence = 0.0;
                        break;
                    }
                }
                None => {
                    // Unknown ingredient: reduce confidence (ambiguous)
                    confidence -= 0.2;
                    if confidence <= 0.0 {
                        confidence = 0.0;
                        break;
                    }
                }
            }
        }

        if confidence > 0.0 {
            tags.push(Tag::new(diet.label, confidence));
        }
    }

    finalize_tags(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_extraction::{ExtractionSource, Ingredient};

    fn make_recipe_with_ingredients(ingredients: Vec<&str>) -> ExtractedRecipe {
        let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        recipe.ingredients = ingredients
            .into_iter()
            .map(|name| Ingredient::new(name, None, None, name))
            .collect();
        recipe
    }

    #[test]
    fn test_all_plant_based_vegan_vegetarian() {
        let recipe =
            make_recipe_with_ingredients(vec!["rice", "olive oil", "tomato", "basil", "salt"]);
        let tags = tag(&recipe);
        let vegan = tags.iter().find(|t| t.label == "vegan");
        let vegetarian = tags.iter().find(|t| t.label == "vegetarian");
        assert!(vegan.is_some(), "Expected vegan tag, got: {:?}", tags);
        assert!(vegetarian.is_some(), "Expected vegetarian tag");
    }

    #[test]
    fn test_wheat_flour_not_gluten_free_sc006() {
        let recipe =
            make_recipe_with_ingredients(vec!["all-purpose flour", "sugar", "butter", "eggs"]);
        let tags = tag(&recipe);
        let gluten_free = tags.iter().find(|t| t.label == "gluten-free");
        assert!(
            gluten_free.is_none(),
            "SC-006: Recipe with wheat flour must NOT be tagged gluten-free. Tags: {:?}",
            tags
        );
    }

    #[test]
    fn test_chicken_no_vegan_vegetarian() {
        let recipe = make_recipe_with_ingredients(vec!["chicken breast", "rice", "broccoli"]);
        let tags = tag(&recipe);
        let vegan = tags.iter().find(|t| t.label == "vegan");
        let vegetarian = tags.iter().find(|t| t.label == "vegetarian");
        assert!(vegan.is_none(), "Chicken recipe should not be vegan");
        assert!(
            vegetarian.is_none(),
            "Chicken recipe should not be vegetarian"
        );
    }

    #[test]
    fn test_butter_ambiguous_reduces_dairy_free_confidence() {
        // Butter is known dairy - should exclude dairy-free entirely
        let recipe = make_recipe_with_ingredients(vec!["rice", "butter", "salt"]);
        let tags = tag(&recipe);
        let dairy_free = tags.iter().find(|t| t.label == "dairy-free");
        assert!(dairy_free.is_none(), "Butter should exclude dairy-free");
    }

    #[test]
    fn test_no_ingredients_fail_safe() {
        let recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        let tags = tag(&recipe);
        assert!(
            tags.is_empty(),
            "No ingredients should yield no diet tags (fail-safe)"
        );
    }

    #[test]
    fn test_ingredient_normalization() {
        // "organic all-purpose flour" should normalize to "wheat flour" -> contains gluten
        let recipe = make_recipe_with_ingredients(vec!["organic all-purpose flour", "sugar"]);
        let tags = tag(&recipe);
        let gluten_free = tags.iter().find(|t| t.label == "gluten-free");
        assert!(
            gluten_free.is_none(),
            "Normalized 'organic all-purpose flour' should flag gluten. Tags: {:?}",
            tags
        );
    }
}
