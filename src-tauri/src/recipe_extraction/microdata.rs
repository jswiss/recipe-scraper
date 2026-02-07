//! Microdata recipe extraction from HTML content.
//!
//! Extracts recipe data from HTML using itemscope/itemprop attributes
//! following the schema.org Recipe vocabulary.

use scraper::{ElementRef, Html, Selector};

use super::duration::parse_duration_flexible;
use super::models::{
    justifications, ExtractedField, ExtractedRecipe, ExtractionError, ExtractionResult,
    ExtractionSource, Ingredient, Instruction, NutritionInfo,
};

/// Extracts recipe data from Microdata in HTML content.
///
/// Searches for elements with `itemscope` and `itemtype` containing "Recipe".
pub fn extract_from_microdata(html: &str) -> ExtractionResult {
    let document = Html::parse_document(html);

    // Find the recipe itemscope
    let recipe_element = find_recipe_itemscope(&document).ok_or_else(|| {
        ExtractionError::no_recipe_found("No Recipe microdata itemscope found", html)
    })?;

    let mut recipe = ExtractedRecipe::empty(ExtractionSource::Microdata);

    // Title (name)
    recipe.title = extract_itemprop_text(&recipe_element, "name");

    // Description
    recipe.description = extract_itemprop_text(&recipe_element, "description");

    // Ingredients
    recipe.ingredients = extract_ingredients_from_microdata(&recipe_element);

    // Instructions
    recipe.instructions = extract_instructions_from_microdata(&recipe_element);

    // Prep time
    recipe.prep_time_minutes = extract_duration_itemprop(&recipe_element, "prepTime");

    // Cook time
    recipe.cook_time_minutes = extract_duration_itemprop(&recipe_element, "cookTime");

    // Servings (recipeYield)
    recipe.servings = extract_itemprop_text(&recipe_element, "recipeYield");

    // Images
    recipe.images = extract_images_from_microdata(&recipe_element);

    // Nutrition
    recipe.nutrition = extract_nutrition_from_microdata(&recipe_element);

    // Verify we got at least some content
    if !recipe.has_content() {
        return Err(ExtractionError::InvalidMicrodata {
            message: "Recipe microdata found but no content could be extracted".to_string(),
        });
    }

    Ok(recipe)
}

/// Finds the recipe itemscope element in the document.
fn find_recipe_itemscope(document: &Html) -> Option<ElementRef<'_>> {
    // Try multiple selector patterns
    let selectors = [
        "[itemscope][itemtype*='schema.org/Recipe']",
        "[itemscope][itemtype*='Recipe']",
        "[itemtype*='schema.org/Recipe']",
    ];

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                return Some(element);
            }
        }
    }

    None
}

/// Extracts the value of an itemprop element.
///
/// Checks for content attribute first (used for datetime, etc.),
/// then falls back to text content.
fn extract_itemprop(element: &ElementRef, prop: &str) -> Option<String> {
    let selector = Selector::parse(&format!("[itemprop='{}']", prop)).ok()?;

    element.select(&selector).next().and_then(|el| {
        // Try content attribute first
        if let Some(content) = el.value().attr("content") {
            if !content.trim().is_empty() {
                return Some(content.trim().to_string());
            }
        }

        // Try datetime attribute (for time elements)
        if let Some(datetime) = el.value().attr("datetime") {
            if !datetime.trim().is_empty() {
                return Some(datetime.trim().to_string());
            }
        }

        // Try value attribute
        if let Some(value) = el.value().attr("value") {
            if !value.trim().is_empty() {
                return Some(value.trim().to_string());
            }
        }

        // Fall back to text content
        let text: String = el.text().collect::<Vec<_>>().join(" ");
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            Some(trimmed.to_string())
        } else {
            None
        }
    })
}

/// Extracts an itemprop as an ExtractedField.
fn extract_itemprop_text(element: &ElementRef, prop: &str) -> ExtractedField<String> {
    match extract_itemprop(element, prop) {
        Some(text) => ExtractedField::found(text),
        None => ExtractedField::not_found(justifications::NOT_FOUND),
    }
}

/// Extracts a duration itemprop and converts to minutes.
fn extract_duration_itemprop(element: &ElementRef, prop: &str) -> ExtractedField<u32> {
    match extract_itemprop(element, prop) {
        Some(duration_str) => match parse_duration_flexible(&duration_str) {
            Some(minutes) => ExtractedField::found(minutes),
            None => ExtractedField::not_found(justifications::PARSE_ERROR),
        },
        None => ExtractedField::not_found(justifications::NOT_FOUND),
    }
}

/// Extracts all ingredients from microdata.
fn extract_ingredients_from_microdata(element: &ElementRef) -> Vec<Ingredient> {
    let mut ingredients = Vec::new();

    if let Ok(selector) = Selector::parse("[itemprop='recipeIngredient']") {
        for el in element.select(&selector) {
            let text: String = el.text().collect::<Vec<_>>().join(" ");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                ingredients.push(Ingredient::from_raw(trimmed));
            }
        }
    }

    // Also try "ingredients" (older schema)
    if ingredients.is_empty() {
        if let Ok(selector) = Selector::parse("[itemprop='ingredients']") {
            for el in element.select(&selector) {
                let text: String = el.text().collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    ingredients.push(Ingredient::from_raw(trimmed));
                }
            }
        }
    }

    ingredients
}

/// Extracts all instructions from microdata.
fn extract_instructions_from_microdata(element: &ElementRef) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    if let Ok(selector) = Selector::parse("[itemprop='recipeInstructions']") {
        for el in element.select(&selector) {
            // Check if this is a HowToStep itemscope
            if el.value().attr("itemscope").is_some() {
                // Extract from nested text itemprop
                if let Some(text) = extract_howto_step_text(&el) {
                    let step_num = (instructions.len() + 1) as u32;
                    instructions.push(Instruction::new(step_num, text));
                }
            } else {
                // Plain text instruction
                let text: String = el.text().collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    // Check if it contains multiple steps (newline-separated)
                    for line in trimmed.lines() {
                        let line_trimmed = line.trim();
                        if !line_trimmed.is_empty() {
                            let step_num = (instructions.len() + 1) as u32;
                            instructions.push(Instruction::new(step_num, line_trimmed));
                        }
                    }
                }
            }
        }
    }

    instructions
}

/// Extracts text from a HowToStep element.
fn extract_howto_step_text(element: &ElementRef) -> Option<String> {
    // Try text itemprop
    if let Ok(selector) = Selector::parse("[itemprop='text']") {
        if let Some(text_el) = element.select(&selector).next() {
            let text: String = text_el.text().collect::<Vec<_>>().join(" ");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Try name itemprop
    if let Ok(selector) = Selector::parse("[itemprop='name']") {
        if let Some(name_el) = element.select(&selector).next() {
            let text: String = name_el.text().collect::<Vec<_>>().join(" ");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Fall back to all text content
    let text: String = element.text().collect::<Vec<_>>().join(" ");
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Extracts images from microdata.
pub fn extract_images_from_microdata(element: &ElementRef) -> ExtractedField<Vec<String>> {
    let mut images = Vec::new();

    if let Ok(selector) = Selector::parse("[itemprop='image']") {
        for el in element.select(&selector) {
            // Try src attribute (img elements)
            if let Some(src) = el.value().attr("src") {
                if !src.trim().is_empty() {
                    images.push(src.trim().to_string());
                    continue;
                }
            }

            // Try href attribute (link elements)
            if let Some(href) = el.value().attr("href") {
                if !href.trim().is_empty() {
                    images.push(href.trim().to_string());
                    continue;
                }
            }

            // Try content attribute
            if let Some(content) = el.value().attr("content") {
                if !content.trim().is_empty() {
                    images.push(content.trim().to_string());
                }
            }
        }
    }

    if images.is_empty() {
        ExtractedField::not_found(justifications::NOT_FOUND)
    } else {
        ExtractedField::found(images)
    }
}

/// Extracts nutrition information from microdata.
pub fn extract_nutrition_from_microdata(element: &ElementRef) -> ExtractedField<NutritionInfo> {
    // Find the nutrition itemscope
    let nutrition_selector = Selector::parse("[itemprop='nutrition']").expect("Invalid selector");

    let nutrition_element = match element.select(&nutrition_selector).next() {
        Some(el) => el,
        None => return ExtractedField::not_found(justifications::NOT_PROVIDED),
    };

    // Parse each nutrition field
    let info = NutritionInfo {
        calories: extract_itemprop(&nutrition_element, "calories")
            .and_then(|s| parse_nutrition_value(&s))
            .map(|v| v as u32),
        fat_grams: extract_itemprop(&nutrition_element, "fatContent")
            .and_then(|s| parse_nutrition_value(&s)),
        saturated_fat_grams: extract_itemprop(&nutrition_element, "saturatedFatContent")
            .and_then(|s| parse_nutrition_value(&s)),
        carbs_grams: extract_itemprop(&nutrition_element, "carbohydrateContent")
            .and_then(|s| parse_nutrition_value(&s)),
        fiber_grams: extract_itemprop(&nutrition_element, "fiberContent")
            .and_then(|s| parse_nutrition_value(&s)),
        sugar_grams: extract_itemprop(&nutrition_element, "sugarContent")
            .and_then(|s| parse_nutrition_value(&s)),
        protein_grams: extract_itemprop(&nutrition_element, "proteinContent")
            .and_then(|s| parse_nutrition_value(&s)),
        sodium_mg: extract_itemprop(&nutrition_element, "sodiumContent")
            .and_then(|s| parse_nutrition_value(&s))
            .map(|v| v as u32),
    };

    if info.has_any_data() {
        ExtractedField::found(info)
    } else {
        ExtractedField::not_found(justifications::PARSE_ERROR)
    }
}

/// Parses a nutrition value from a string like "100 g" or "100".
fn parse_nutrition_value(s: &str) -> Option<f64> {
    let num_str: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MICRODATA_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div itemscope itemtype="https://schema.org/Recipe">
                <h1 itemprop="name">Chocolate Chip Cookies</h1>
                <p itemprop="description">Classic homemade cookies</p>
                <ul>
                    <li itemprop="recipeIngredient">2 cups flour</li>
                    <li itemprop="recipeIngredient">1 cup butter</li>
                </ul>
                <meta itemprop="prepTime" content="PT15M">
                <meta itemprop="cookTime" content="PT10M">
                <span itemprop="recipeYield">24 cookies</span>
                <img itemprop="image" src="https://example.com/cookies.jpg">
                <div itemprop="recipeInstructions">
                    Preheat oven to 375°F.
                    Mix ingredients.
                </div>
            </div>
        </body>
        </html>
    "#;

    #[test]
    fn test_extract_from_microdata() {
        let result = extract_from_microdata(SAMPLE_MICRODATA_HTML);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(recipe.source, ExtractionSource::Microdata);
        assert_eq!(
            recipe.title,
            ExtractedField::found("Chocolate Chip Cookies".to_string())
        );
        assert_eq!(
            recipe.description,
            ExtractedField::found("Classic homemade cookies".to_string())
        );
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.prep_time_minutes, ExtractedField::found(15));
        assert_eq!(recipe.cook_time_minutes, ExtractedField::found(10));
        assert_eq!(
            recipe.servings,
            ExtractedField::found("24 cookies".to_string())
        );
    }

    #[test]
    fn test_find_recipe_itemscope() {
        let document = Html::parse_document(SAMPLE_MICRODATA_HTML);
        let element = find_recipe_itemscope(&document);
        assert!(element.is_some());
    }

    #[test]
    fn test_no_microdata() {
        let html = "<html><body><p>No recipe here</p></body></html>";
        let result = extract_from_microdata(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtractionError::NoRecipeFound { .. }
        ));
    }

    #[test]
    fn test_extract_ingredients() {
        let document = Html::parse_document(SAMPLE_MICRODATA_HTML);
        let element = find_recipe_itemscope(&document).unwrap();
        let ingredients = extract_ingredients_from_microdata(&element);
        assert_eq!(ingredients.len(), 2);
        assert_eq!(ingredients[0].raw_text, "2 cups flour");
        assert_eq!(ingredients[1].raw_text, "1 cup butter");
    }

    #[test]
    fn test_extract_image_from_src() {
        let document = Html::parse_document(SAMPLE_MICRODATA_HTML);
        let element = find_recipe_itemscope(&document).unwrap();
        let images = extract_images_from_microdata(&element);
        assert!(images.is_found());
        if let ExtractedField::Found { value } = images {
            assert!(value.contains(&"https://example.com/cookies.jpg".to_string()));
        }
    }

    #[test]
    fn test_howto_step_microdata() {
        let html = r#"
            <div itemscope itemtype="https://schema.org/Recipe">
                <h1 itemprop="name">Test</h1>
                <div itemprop="recipeInstructions" itemscope itemtype="https://schema.org/HowToStep">
                    <span itemprop="text">Step one text</span>
                </div>
                <div itemprop="recipeInstructions" itemscope itemtype="https://schema.org/HowToStep">
                    <span itemprop="text">Step two text</span>
                </div>
            </div>
        "#;

        let result = extract_from_microdata(html).unwrap();
        assert_eq!(result.instructions.len(), 2);
        assert_eq!(result.instructions[0].text, "Step one text");
        assert_eq!(result.instructions[1].text, "Step two text");
    }
}
