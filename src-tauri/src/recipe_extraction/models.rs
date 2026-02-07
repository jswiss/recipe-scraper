//! Data models for recipe extraction.
//!
//! All types follow the schema.org Recipe vocabulary and ensure every field
//! is either present with a value or explicitly null with justification.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Standard justification messages for missing fields.
pub mod justifications {
    pub const NOT_FOUND: &str = "Not found in source";
    pub const NOT_PROVIDED: &str = "Not provided";
    #[allow(dead_code)]
    pub const AMBIGUOUS: &str = "Value was ambiguous";
    pub const PARSE_ERROR: &str = "Could not parse value";
}

/// A field that is either found with a value or not found with a justification.
///
/// This ensures every field in the recipe response is explicitly accounted for,
/// making it clear to consumers what was extracted vs what was missing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExtractedField<T> {
    /// Field was successfully extracted
    Found { value: T },
    /// Field could not be extracted
    NotFound { justification: String },
}

impl<T> ExtractedField<T> {
    /// Creates a Found variant with the given value.
    pub fn found(value: T) -> Self {
        ExtractedField::Found { value }
    }

    /// Creates a NotFound variant with the given justification.
    pub fn not_found(justification: impl Into<String>) -> Self {
        ExtractedField::NotFound {
            justification: justification.into(),
        }
    }

    /// Returns true if this field has a value.
    pub fn is_found(&self) -> bool {
        matches!(self, ExtractedField::Found { .. })
    }

    /// Returns the value if found, or None if not found.
    pub fn value(&self) -> Option<&T> {
        match self {
            ExtractedField::Found { value } => Some(value),
            ExtractedField::NotFound { .. } => None,
        }
    }
}

/// Indicates how the recipe data was extracted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionSource {
    /// Extracted from JSON-LD structured data
    JsonLd,
    /// Extracted from Microdata (itemscope/itemprop)
    Microdata,
    /// Extracted using local AI model from HTML content
    AiFallback,
}

/// A recipe ingredient with optional structured data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ingredient {
    /// Ingredient name (e.g., "all-purpose flour")
    pub name: String,
    /// Quantity if parseable (e.g., 2.0, 0.5)
    pub quantity: Option<f64>,
    /// Unit if parseable (e.g., "cups", "tablespoons")
    pub unit: Option<String>,
    /// Original raw text from source
    pub raw_text: String,
}

impl Ingredient {
    /// Creates a new ingredient from raw text, using the text as both name and raw_text.
    pub fn from_raw(raw_text: impl Into<String>) -> Self {
        let text = raw_text.into();
        Self {
            name: text.clone(),
            quantity: None,
            unit: None,
            raw_text: text,
        }
    }

    /// Creates a new ingredient with structured data.
    pub fn new(
        name: impl Into<String>,
        quantity: Option<f64>,
        unit: Option<String>,
        raw_text: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            quantity,
            unit,
            raw_text: raw_text.into(),
        }
    }
}

/// A single step in the recipe preparation process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instruction {
    /// 1-indexed step number
    pub step_number: u32,
    /// Instruction text
    pub text: String,
}

impl Instruction {
    /// Creates a new instruction.
    pub fn new(step_number: u32, text: impl Into<String>) -> Self {
        Self {
            step_number,
            text: text.into(),
        }
    }
}

/// Nutritional information following schema.org NutritionInformation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NutritionInfo {
    /// Calories per serving
    pub calories: Option<u32>,
    /// Total fat in grams
    pub fat_grams: Option<f64>,
    /// Saturated fat in grams
    pub saturated_fat_grams: Option<f64>,
    /// Total carbohydrates in grams
    pub carbs_grams: Option<f64>,
    /// Dietary fiber in grams
    pub fiber_grams: Option<f64>,
    /// Sugar in grams
    pub sugar_grams: Option<f64>,
    /// Protein in grams
    pub protein_grams: Option<f64>,
    /// Sodium in milligrams
    pub sodium_mg: Option<u32>,
}

impl NutritionInfo {
    /// Returns true if any nutrition field has a value.
    pub fn has_any_data(&self) -> bool {
        self.calories.is_some()
            || self.fat_grams.is_some()
            || self.saturated_fat_grams.is_some()
            || self.carbs_grams.is_some()
            || self.fiber_grams.is_some()
            || self.sugar_grams.is_some()
            || self.protein_grams.is_some()
            || self.sodium_mg.is_some()
    }
}

/// The complete extracted recipe containing all fields.
///
/// Every field is present with valid content or marked explicitly null with justification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedRecipe {
    /// Recipe title
    pub title: ExtractedField<String>,
    /// Recipe description/summary
    pub description: ExtractedField<String>,
    /// List of ingredients
    pub ingredients: Vec<Ingredient>,
    /// Ordered list of preparation steps
    pub instructions: Vec<Instruction>,
    /// Preparation time in minutes
    pub prep_time_minutes: ExtractedField<u32>,
    /// Cooking time in minutes
    pub cook_time_minutes: ExtractedField<u32>,
    /// Serving size/yield (e.g., "4 servings", "12 cookies")
    pub servings: ExtractedField<String>,
    /// Image URLs (absolute URLs)
    pub images: ExtractedField<Vec<String>>,
    /// Nutritional information
    pub nutrition: ExtractedField<NutritionInfo>,
    /// How the recipe was extracted
    pub source: ExtractionSource,
}

impl ExtractedRecipe {
    /// Creates a new ExtractedRecipe with all fields set to NotFound.
    pub fn empty(source: ExtractionSource) -> Self {
        Self {
            title: ExtractedField::not_found(justifications::NOT_FOUND),
            description: ExtractedField::not_found(justifications::NOT_FOUND),
            ingredients: Vec::new(),
            instructions: Vec::new(),
            prep_time_minutes: ExtractedField::not_found(justifications::NOT_FOUND),
            cook_time_minutes: ExtractedField::not_found(justifications::NOT_FOUND),
            servings: ExtractedField::not_found(justifications::NOT_FOUND),
            images: ExtractedField::not_found(justifications::NOT_FOUND),
            nutrition: ExtractedField::not_found(justifications::NOT_FOUND),
            source,
        }
    }

    /// Returns true if the recipe has minimal viable content (title, ingredients, or instructions).
    pub fn has_content(&self) -> bool {
        self.title.is_found() || !self.ingredients.is_empty() || !self.instructions.is_empty()
    }
}

/// Errors that can occur during recipe extraction.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum ExtractionError {
    /// No recipe found in the HTML content
    #[error("No recipe found: {message}")]
    NoRecipeFound {
        message: String,
        html_preview: String,
    },

    /// JSON-LD parsing failed
    #[error("Invalid JSON-LD: {message}")]
    InvalidJsonLd { message: String, raw_json: String },

    /// Microdata parsing failed
    #[error("Invalid Microdata: {message}")]
    InvalidMicrodata { message: String },

    /// Local AI model failed
    #[error("AI extraction failed: {message}")]
    AiExtractionFailed { message: String },

    /// Model not available (not downloaded yet)
    #[error("AI model not available: {message}")]
    ModelNotAvailable { message: String },
}

impl ExtractionError {
    /// Creates a NoRecipeFound error with HTML preview.
    pub fn no_recipe_found(message: impl Into<String>, html: &str) -> Self {
        let preview = if html.len() > 200 {
            format!("{}...", &html[..200])
        } else {
            html.to_string()
        };
        Self::NoRecipeFound {
            message: message.into(),
            html_preview: preview,
        }
    }
}

/// Result type for recipe extraction operations.
pub type ExtractionResult = Result<ExtractedRecipe, ExtractionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracted_field_found() {
        let field: ExtractedField<String> = ExtractedField::found("test".to_string());
        assert!(field.is_found());
        assert_eq!(field.value(), Some(&"test".to_string()));
    }

    #[test]
    fn test_extracted_field_not_found() {
        let field: ExtractedField<String> = ExtractedField::not_found("Not available");
        assert!(!field.is_found());
        assert_eq!(field.value(), None);
    }

    #[test]
    fn test_ingredient_from_raw() {
        let ingredient = Ingredient::from_raw("2 cups flour");
        assert_eq!(ingredient.name, "2 cups flour");
        assert_eq!(ingredient.raw_text, "2 cups flour");
        assert!(ingredient.quantity.is_none());
        assert!(ingredient.unit.is_none());
    }

    #[test]
    fn test_nutrition_info_has_any_data() {
        let empty = NutritionInfo::default();
        assert!(!empty.has_any_data());

        let with_calories = NutritionInfo {
            calories: Some(100),
            ..Default::default()
        };
        assert!(with_calories.has_any_data());
    }

    #[test]
    fn test_extracted_recipe_has_content() {
        let empty = ExtractedRecipe::empty(ExtractionSource::JsonLd);
        assert!(!empty.has_content());

        let with_title = ExtractedRecipe {
            title: ExtractedField::found("Test Recipe".to_string()),
            ..ExtractedRecipe::empty(ExtractionSource::JsonLd)
        };
        assert!(with_title.has_content());
    }

    #[test]
    fn test_serialization() {
        let field: ExtractedField<String> = ExtractedField::found("test".to_string());
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("\"status\":\"found\""));
        assert!(json.contains("\"value\":\"test\""));

        let not_found: ExtractedField<String> = ExtractedField::not_found("missing");
        let json = serde_json::to_string(&not_found).unwrap();
        assert!(json.contains("\"status\":\"not_found\""));
        assert!(json.contains("\"justification\":\"missing\""));
    }
}
