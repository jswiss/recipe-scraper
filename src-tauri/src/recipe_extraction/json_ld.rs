//! JSON-LD recipe extraction from HTML content.
//!
//! Extracts recipe data from `<script type="application/ld+json">` tags
//! following the schema.org Recipe vocabulary.

use scraper::{Html, Selector};
use serde_json::Value;

use super::duration::parse_duration_flexible;
use super::models::{
    justifications, ExtractedField, ExtractedRecipe, ExtractionError, ExtractionResult,
    ExtractionSource, Ingredient, Instruction, NutritionInfo,
};

/// Extracts recipe data from JSON-LD in HTML content.
///
/// Searches for `<script type="application/ld+json">` tags and parses
/// any that contain a Recipe schema.
pub fn extract_from_jsonld(html: &str) -> ExtractionResult {
    let scripts = find_jsonld_scripts(html);

    if scripts.is_empty() {
        return Err(ExtractionError::no_recipe_found(
            "No JSON-LD scripts found",
            html,
        ));
    }

    for script_content in scripts {
        match serde_json::from_str::<Value>(&script_content) {
            Ok(value) => {
                if let Some(recipe_value) = find_recipe_in_value(&value) {
                    return parse_recipe_from_jsonld(recipe_value);
                }
            }
            Err(e) => {
                log::debug!("Failed to parse JSON-LD: {}", e);
                continue;
            }
        }
    }

    Err(ExtractionError::no_recipe_found(
        "No Recipe schema found in JSON-LD",
        html,
    ))
}

/// Finds all JSON-LD script contents in the HTML.
fn find_jsonld_scripts(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script[type='application/ld+json']").expect("Invalid selector");

    document
        .select(&selector)
        .map(|el| el.text().collect::<String>())
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// Recursively searches for a Recipe schema in a JSON value.
///
/// Handles:
/// - Direct Recipe object
/// - @graph arrays containing Recipe objects
/// - Arrays of schema objects
fn find_recipe_in_value(value: &Value) -> Option<&Value> {
    // Check if this value is a Recipe
    if is_recipe_schema(value) {
        return Some(value);
    }

    // Check @graph array
    if let Some(graph) = value.get("@graph") {
        if let Some(arr) = graph.as_array() {
            for item in arr {
                if is_recipe_schema(item) {
                    return Some(item);
                }
            }
        }
    }

    // Check if it's an array of schemas
    if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(recipe) = find_recipe_in_value(item) {
                return Some(recipe);
            }
        }
    }

    None
}

/// Checks if a JSON value represents a Recipe schema.
fn is_recipe_schema(value: &Value) -> bool {
    match value.get("@type") {
        Some(Value::String(t)) => t == "Recipe" || t.ends_with("/Recipe"),
        Some(Value::Array(arr)) => arr.iter().any(|v| {
            v.as_str()
                .map(|s| s == "Recipe" || s.ends_with("/Recipe"))
                .unwrap_or(false)
        }),
        _ => false,
    }
}

/// Parses a Recipe JSON-LD value into an ExtractedRecipe.
fn parse_recipe_from_jsonld(value: &Value) -> ExtractionResult {
    let mut recipe = ExtractedRecipe::empty(ExtractionSource::JsonLd);

    // Title (name)
    recipe.title = extract_string_field(value, "name");

    // Description
    recipe.description = extract_string_field(value, "description");

    // Ingredients
    recipe.ingredients = extract_ingredients_from_jsonld(value);

    // Instructions
    recipe.instructions = extract_instructions_from_jsonld(value);

    // Prep time
    recipe.prep_time_minutes = extract_duration_field(value, "prepTime");

    // Cook time
    recipe.cook_time_minutes = extract_duration_field(value, "cookTime");

    // Servings (recipeYield)
    recipe.servings = extract_yield_field(value);

    // Images
    recipe.images = extract_images_from_jsonld(value);

    // Nutrition
    recipe.nutrition = extract_nutrition_from_jsonld(value);

    // Verify we got at least some content
    if !recipe.has_content() {
        return Err(ExtractionError::InvalidJsonLd {
            message: "Recipe schema found but no content could be extracted".to_string(),
            raw_json: value.to_string(),
        });
    }

    Ok(recipe)
}

/// Extracts a string field from a JSON value.
fn extract_string_field(value: &Value, field: &str) -> ExtractedField<String> {
    match value.get(field) {
        Some(Value::String(s)) if !s.trim().is_empty() => {
            ExtractedField::found(s.trim().to_string())
        }
        Some(Value::Array(arr)) => {
            // Some fields might be arrays; take the first string
            for item in arr {
                if let Some(s) = item.as_str() {
                    if !s.trim().is_empty() {
                        return ExtractedField::found(s.trim().to_string());
                    }
                }
            }
            ExtractedField::not_found(justifications::NOT_FOUND)
        }
        _ => ExtractedField::not_found(justifications::NOT_FOUND),
    }
}

/// Extracts a duration field and converts to minutes.
fn extract_duration_field(value: &Value, field: &str) -> ExtractedField<u32> {
    match value.get(field) {
        Some(Value::String(s)) => match parse_duration_flexible(s) {
            Some(minutes) => ExtractedField::found(minutes),
            None => ExtractedField::not_found(justifications::PARSE_ERROR),
        },
        Some(Value::Number(n)) => match n.as_u64() {
            Some(minutes) => ExtractedField::found(minutes as u32),
            None => ExtractedField::not_found(justifications::PARSE_ERROR),
        },
        _ => ExtractedField::not_found(justifications::NOT_FOUND),
    }
}

/// Extracts the recipe yield/servings field.
fn extract_yield_field(value: &Value) -> ExtractedField<String> {
    // Try recipeYield first
    if let Some(yield_val) = value.get("recipeYield") {
        match yield_val {
            Value::String(s) if !s.trim().is_empty() => {
                return ExtractedField::found(s.trim().to_string())
            }
            Value::Number(n) => return ExtractedField::found(n.to_string()),
            Value::Array(arr) => {
                // Take the first non-empty value
                for item in arr {
                    match item {
                        Value::String(s) if !s.trim().is_empty() => {
                            return ExtractedField::found(s.trim().to_string())
                        }
                        Value::Number(n) => return ExtractedField::found(n.to_string()),
                        _ => continue,
                    }
                }
            }
            _ => {}
        }
    }

    // Try yield as fallback
    if let Some(Value::String(s)) = value.get("yield") {
        if !s.trim().is_empty() {
            return ExtractedField::found(s.trim().to_string());
        }
    }

    ExtractedField::not_found(justifications::NOT_FOUND)
}

/// Extracts ingredients from JSON-LD recipeIngredient field.
fn extract_ingredients_from_jsonld(value: &Value) -> Vec<Ingredient> {
    let mut ingredients = Vec::new();

    if let Some(Value::Array(arr)) = value.get("recipeIngredient") {
        for item in arr {
            if let Some(text) = item.as_str() {
                if !text.trim().is_empty() {
                    ingredients.push(Ingredient::from_raw(text.trim()));
                }
            }
        }
    }

    ingredients
}

/// Extracts instructions from JSON-LD recipeInstructions field.
///
/// Handles both string arrays and HowToStep/HowToSection objects.
fn extract_instructions_from_jsonld(value: &Value) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    match value.get("recipeInstructions") {
        Some(Value::Array(arr)) => {
            let mut step_num = 1;
            for item in arr {
                match item {
                    // Plain string instruction
                    Value::String(s) if !s.trim().is_empty() => {
                        instructions.push(Instruction::new(step_num, s.trim()));
                        step_num += 1;
                    }
                    // HowToStep or HowToSection object
                    Value::Object(_) => {
                        if let Some(text) = extract_howto_text(item) {
                            instructions.push(Instruction::new(step_num, text));
                            step_num += 1;
                        }
                        // Handle HowToSection with itemListElement
                        if let Some(Value::Array(items)) = item.get("itemListElement") {
                            for sub_item in items {
                                if let Some(text) = extract_howto_text(sub_item) {
                                    instructions.push(Instruction::new(step_num, text));
                                    step_num += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        // Single string instruction
        Some(Value::String(s)) if !s.trim().is_empty() => {
            // Split by newlines or numbered patterns
            for (i, line) in s.lines().enumerate() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    instructions.push(Instruction::new((i + 1) as u32, trimmed));
                }
            }
        }
        _ => {}
    }

    instructions
}

/// Extracts text from a HowToStep or similar object.
fn extract_howto_text(value: &Value) -> Option<String> {
    // Try "text" field first
    if let Some(Value::String(s)) = value.get("text") {
        if !s.trim().is_empty() {
            return Some(s.trim().to_string());
        }
    }

    // Try "name" field as fallback
    if let Some(Value::String(s)) = value.get("name") {
        if !s.trim().is_empty() {
            return Some(s.trim().to_string());
        }
    }

    None
}

/// Extracts images from JSON-LD image field.
///
/// Handles string, array of strings, and ImageObject formats.
pub fn extract_images_from_jsonld(value: &Value) -> ExtractedField<Vec<String>> {
    let mut images = Vec::new();

    match value.get("image") {
        Some(Value::String(s)) if !s.trim().is_empty() => {
            images.push(s.trim().to_string());
        }
        Some(Value::Array(arr)) => {
            for item in arr {
                match item {
                    Value::String(s) if !s.trim().is_empty() => {
                        images.push(s.trim().to_string());
                    }
                    Value::Object(_) => {
                        // ImageObject - try url or contentUrl
                        if let Some(url) = item
                            .get("url")
                            .or_else(|| item.get("contentUrl"))
                            .and_then(|v| v.as_str())
                        {
                            if !url.trim().is_empty() {
                                images.push(url.trim().to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Some(Value::Object(obj)) => {
            // Single ImageObject
            if let Some(url) = obj
                .get("url")
                .or_else(|| obj.get("contentUrl"))
                .and_then(|v| v.as_str())
            {
                if !url.trim().is_empty() {
                    images.push(url.trim().to_string());
                }
            }
        }
        _ => {}
    }

    if images.is_empty() {
        ExtractedField::not_found(justifications::NOT_FOUND)
    } else {
        ExtractedField::found(images)
    }
}

/// Extracts nutrition information from JSON-LD nutrition field.
pub fn extract_nutrition_from_jsonld(value: &Value) -> ExtractedField<NutritionInfo> {
    let nutrition_value = match value.get("nutrition") {
        Some(v) => v,
        None => return ExtractedField::not_found(justifications::NOT_PROVIDED),
    };

    let mut info = NutritionInfo::default();

    // Parse calories (can be "100 calories" or just number)
    if let Some(cal) = nutrition_value.get("calories") {
        info.calories = parse_nutrition_value(cal).map(|v| v as u32);
    }

    // Parse fat content
    if let Some(fat) = nutrition_value.get("fatContent") {
        info.fat_grams = parse_nutrition_value(fat);
    }

    // Parse saturated fat
    if let Some(sat_fat) = nutrition_value.get("saturatedFatContent") {
        info.saturated_fat_grams = parse_nutrition_value(sat_fat);
    }

    // Parse carbohydrates
    if let Some(carbs) = nutrition_value.get("carbohydrateContent") {
        info.carbs_grams = parse_nutrition_value(carbs);
    }

    // Parse fiber
    if let Some(fiber) = nutrition_value.get("fiberContent") {
        info.fiber_grams = parse_nutrition_value(fiber);
    }

    // Parse sugar
    if let Some(sugar) = nutrition_value.get("sugarContent") {
        info.sugar_grams = parse_nutrition_value(sugar);
    }

    // Parse protein
    if let Some(protein) = nutrition_value.get("proteinContent") {
        info.protein_grams = parse_nutrition_value(protein);
    }

    // Parse sodium
    if let Some(sodium) = nutrition_value.get("sodiumContent") {
        info.sodium_mg = parse_nutrition_value(sodium).map(|v| v as u32);
    }

    if info.has_any_data() {
        ExtractedField::found(info)
    } else {
        ExtractedField::not_found(justifications::PARSE_ERROR)
    }
}

/// Parses a nutrition value that can be a number or string like "100 g".
fn parse_nutrition_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => {
            // Extract first number from string like "100 g" or "100 calories"
            let num_str: String = s
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            num_str.parse().ok()
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSONLD_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "Recipe",
                "name": "Chocolate Chip Cookies",
                "description": "Classic homemade cookies",
                "recipeIngredient": [
                    "2 cups flour",
                    "1 cup butter"
                ],
                "recipeInstructions": [
                    "Preheat oven to 375°F",
                    "Mix ingredients"
                ],
                "prepTime": "PT15M",
                "cookTime": "PT10M",
                "recipeYield": "24 cookies",
                "image": "https://example.com/cookies.jpg"
            }
            </script>
        </head>
        <body></body>
        </html>
    "#;

    #[test]
    fn test_extract_from_jsonld() {
        let result = extract_from_jsonld(SAMPLE_JSONLD_HTML);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(recipe.source, ExtractionSource::JsonLd);
        assert_eq!(
            recipe.title,
            ExtractedField::found("Chocolate Chip Cookies".to_string())
        );
        assert_eq!(
            recipe.description,
            ExtractedField::found("Classic homemade cookies".to_string())
        );
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.instructions.len(), 2);
        assert_eq!(recipe.prep_time_minutes, ExtractedField::found(15));
        assert_eq!(recipe.cook_time_minutes, ExtractedField::found(10));
        assert_eq!(
            recipe.servings,
            ExtractedField::found("24 cookies".to_string())
        );
    }

    #[test]
    fn test_is_recipe_schema() {
        let recipe: Value = serde_json::from_str(r#"{"@type": "Recipe"}"#).unwrap();
        assert!(is_recipe_schema(&recipe));

        let recipe_array: Value =
            serde_json::from_str(r#"{"@type": ["Recipe", "Thing"]}"#).unwrap();
        assert!(is_recipe_schema(&recipe_array));

        let not_recipe: Value = serde_json::from_str(r#"{"@type": "Person"}"#).unwrap();
        assert!(!is_recipe_schema(&not_recipe));
    }

    #[test]
    fn test_find_jsonld_scripts() {
        let scripts = find_jsonld_scripts(SAMPLE_JSONLD_HTML);
        assert_eq!(scripts.len(), 1);
        assert!(scripts[0].contains("Chocolate Chip Cookies"));
    }

    #[test]
    fn test_extract_with_graph() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@graph": [
                    {"@type": "WebPage"},
                    {"@type": "Recipe", "name": "Test Recipe"}
                ]
            }
            </script>
        "#;

        let result = extract_from_jsonld(html);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().title,
            ExtractedField::found("Test Recipe".to_string())
        );
    }

    #[test]
    fn test_no_recipe_found() {
        let html = r#"
            <script type="application/ld+json">
            {"@type": "Person", "name": "John"}
            </script>
        "#;

        let result = extract_from_jsonld(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtractionError::NoRecipeFound { .. }
        ));
    }

    #[test]
    fn test_howto_instructions() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@type": "Recipe",
                "name": "Test",
                "recipeInstructions": [
                    {"@type": "HowToStep", "text": "Step one"},
                    {"@type": "HowToStep", "text": "Step two"}
                ]
            }
            </script>
        "#;

        let result = extract_from_jsonld(html).unwrap();
        assert_eq!(result.instructions.len(), 2);
        assert_eq!(result.instructions[0].text, "Step one");
        assert_eq!(result.instructions[1].text, "Step two");
    }

    #[test]
    fn test_jsonld_missing_prep_time_returns_not_found() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@type": "Recipe",
                "name": "Test",
                "recipeIngredient": ["flour"],
                "cookTime": "PT10M"
            }
            </script>
        "#;
        let result = extract_from_jsonld(html).unwrap();
        assert!(matches!(
            result.prep_time_minutes,
            ExtractedField::NotFound { .. }
        ));
        if let ExtractedField::NotFound { justification } = &result.prep_time_minutes {
            assert!(!justification.is_empty());
        }
    }

    #[test]
    fn test_jsonld_missing_nutrition_returns_not_found() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@type": "Recipe",
                "name": "Test",
                "recipeIngredient": ["flour"]
            }
            </script>
        "#;
        let result = extract_from_jsonld(html).unwrap();
        assert!(matches!(result.nutrition, ExtractedField::NotFound { .. }));
    }

    #[test]
    fn test_jsonld_missing_images_returns_not_found() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@type": "Recipe",
                "name": "Test",
                "recipeIngredient": ["flour"]
            }
            </script>
        "#;
        let result = extract_from_jsonld(html).unwrap();
        assert!(matches!(result.images, ExtractedField::NotFound { .. }));
    }

    #[test]
    fn test_jsonld_multiple_images_all_captured() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@type": "Recipe",
                "name": "Test",
                "recipeIngredient": ["flour"],
                "image": ["url1.jpg", "url2.jpg", "url3.jpg"]
            }
            </script>
        "#;
        let result = extract_from_jsonld(html).unwrap();
        assert!(result.images.is_found());
        if let ExtractedField::Found { value } = &result.images {
            assert_eq!(value.len(), 3);
            assert!(value.contains(&"url1.jpg".to_string()));
            assert!(value.contains(&"url2.jpg".to_string()));
            assert!(value.contains(&"url3.jpg".to_string()));
        }
    }

    #[test]
    fn test_jsonld_nutrition_fields_extracted() {
        let html = r#"
            <script type="application/ld+json">
            {
                "@type": "Recipe",
                "name": "Test",
                "recipeIngredient": ["flour"],
                "nutrition": {
                    "@type": "NutritionInformation",
                    "calories": "200 calories",
                    "fatContent": "10 g",
                    "carbohydrateContent": "25 g",
                    "proteinContent": "5 g"
                }
            }
            </script>
        "#;
        let result = extract_from_jsonld(html).unwrap();
        assert!(result.nutrition.is_found());
        if let ExtractedField::Found { value } = &result.nutrition {
            assert_eq!(value.calories, Some(200));
            assert_eq!(value.fat_grams, Some(10.0));
            assert_eq!(value.carbs_grams, Some(25.0));
            assert_eq!(value.protein_grams, Some(5.0));
        }
    }

    #[test]
    fn test_jsonld_malformed_falls_back_gracefully() {
        let html = r#"<script type="application/ld+json">{ "broken": }</script>"#;
        let result = extract_from_jsonld(html);
        assert!(result.is_err());
    }
}
