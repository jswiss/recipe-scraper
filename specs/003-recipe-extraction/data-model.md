# Data Model: Recipe Structure Extraction

**Feature**: 003-recipe-extraction
**Date**: 2026-02-04

---

## Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ExtractionResult                              │
│  ├── Ok(ExtractedRecipe)                                            │
│  └── Err(ExtractionError)                                           │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        ExtractedRecipe                               │
│  ├── title: ExtractedField<String>                                  │
│  ├── description: ExtractedField<String>                            │
│  ├── ingredients: Vec<Ingredient>                                   │
│  ├── instructions: Vec<Instruction>                                 │
│  ├── prep_time_minutes: ExtractedField<u32>                        │
│  ├── cook_time_minutes: ExtractedField<u32>                        │
│  ├── servings: ExtractedField<String>                               │
│  ├── images: ExtractedField<Vec<String>>                            │
│  ├── nutrition: ExtractedField<NutritionInfo>                       │
│  └── source: ExtractionSource                                       │
└─────────────────────────────────────────────────────────────────────┘
           │                    │                      │
           ▼                    ▼                      ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐
│    Ingredient    │  │   Instruction    │  │   NutritionInfo      │
│  ├── name        │  │  ├── step_number │  │  ├── calories        │
│  ├── quantity    │  │  └── text        │  │  ├── fat_grams       │
│  ├── unit        │  └──────────────────┘  │  ├── protein_grams   │
│  └── raw_text    │                        │  ├── carbs_grams     │
└──────────────────┘                        │  ├── fiber_grams     │
                                            │  ├── sugar_grams     │
                                            │  └── sodium_mg       │
                                            └──────────────────────┘
```

---

## Core Entities

### ExtractedRecipe

The complete extracted recipe containing all fields. Every field is present with valid content or marked explicitly null with justification.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRecipe {
    /// Recipe title (required field, but may be null if not found)
    pub title: ExtractedField<String>,

    /// Recipe description/summary
    pub description: ExtractedField<String>,

    /// List of ingredients (may be empty with justification)
    pub ingredients: Vec<Ingredient>,

    /// Ordered list of preparation steps (may be empty with justification)
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
```

**Validation Rules**:
- All fields must be present in every response (FR-011)
- Null fields must include justification (FR-012)
- At minimum, `title`, `ingredients`, or `instructions` should have value for valid recipe

---

### ExtractedField<T>

A wrapper type that ensures every field either has a value or an explicit justification for why it's null.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExtractedField<T> {
    /// Field was successfully extracted
    Found { value: T },

    /// Field could not be extracted
    NotFound { justification: String },
}
```

**Usage Examples**:
```json
// Found value
{ "status": "found", "value": "Chocolate Chip Cookies" }

// Not found with justification
{ "status": "not_found", "justification": "No prep time specified in source" }
```

---

### Ingredient

A recipe ingredient with optional structured data or raw text fallback.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

**Validation Rules**:
- `name` is required (extracted from raw_text if not structured)
- `quantity` and `unit` are optional (some ingredients don't have them: "salt to taste")
- `raw_text` always preserves original source text

**State Transitions**: N/A (immutable value object)

---

### Instruction

A single step in the recipe preparation process.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// 1-indexed step number
    pub step_number: u32,

    /// Instruction text
    pub text: String,
}
```

**Validation Rules**:
- `step_number` starts at 1
- `text` is non-empty
- Order matches source document

---

### NutritionInfo

Nutritional data following schema.org NutritionInformation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

**Validation Rules**:
- All fields are optional (partial nutrition data is common)
- Values should be non-negative
- Units are standardized (grams for macros, mg for sodium, kcal for calories)

---

### ExtractionSource

Indicates how the recipe data was extracted.

```rust
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
```

---

### ExtractionError

Errors that can occur during recipe extraction.

```rust
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum ExtractionError {
    /// No recipe found in the HTML content
    #[error("No recipe found: {message}")]
    NoRecipeFound { message: String, html_preview: String },

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
```

---

## Type Aliases

```rust
/// Result type for recipe extraction operations
pub type ExtractionResult = Result<ExtractedRecipe, ExtractionError>;
```

---

## Field Mapping from Schema.org

| Schema.org Property | Entity Field | Transformation |
|---------------------|--------------|----------------|
| `name` | `title` | Direct mapping |
| `description` | `description` | Direct mapping |
| `recipeIngredient` | `ingredients` | Parse to Ingredient structs |
| `recipeInstructions` | `instructions` | Parse HowToStep or strings |
| `prepTime` | `prep_time_minutes` | ISO 8601 duration to minutes |
| `cookTime` | `cook_time_minutes` | ISO 8601 duration to minutes |
| `recipeYield` | `servings` | Direct mapping (text) |
| `image` | `images` | Normalize to URL array |
| `nutrition` | `nutrition` | Map NutritionInformation |

---

## JSON Serialization Examples

### Successful Extraction (JSON-LD Source)

```json
{
  "title": { "status": "found", "value": "Chocolate Chip Cookies" },
  "description": { "status": "found", "value": "Classic homemade cookies" },
  "ingredients": [
    {
      "name": "all-purpose flour",
      "quantity": 2.25,
      "unit": "cups",
      "raw_text": "2 1/4 cups all-purpose flour"
    },
    {
      "name": "butter",
      "quantity": 1.0,
      "unit": "cup",
      "raw_text": "1 cup butter, softened"
    }
  ],
  "instructions": [
    { "step_number": 1, "text": "Preheat oven to 375°F" },
    { "step_number": 2, "text": "Combine flour, baking soda and salt" }
  ],
  "prep_time_minutes": { "status": "found", "value": 15 },
  "cook_time_minutes": { "status": "found", "value": 10 },
  "servings": { "status": "found", "value": "48 cookies" },
  "images": { "status": "found", "value": ["https://example.com/cookies.jpg"] },
  "nutrition": {
    "status": "found",
    "value": {
      "calories": 150,
      "fat_grams": 7.0,
      "protein_grams": 2.0,
      "carbs_grams": 20.0,
      "fiber_grams": null,
      "sugar_grams": 12.0,
      "saturated_fat_grams": 4.5,
      "sodium_mg": 85
    }
  },
  "source": "json_ld"
}
```

### Partial Extraction with Nulls

```json
{
  "title": { "status": "found", "value": "Quick Pasta" },
  "description": { "status": "not_found", "justification": "No description in source" },
  "ingredients": [
    { "name": "pasta", "quantity": null, "unit": null, "raw_text": "pasta" }
  ],
  "instructions": [
    { "step_number": 1, "text": "Boil pasta and serve" }
  ],
  "prep_time_minutes": { "status": "not_found", "justification": "Not specified in source" },
  "cook_time_minutes": { "status": "found", "value": 10 },
  "servings": { "status": "not_found", "justification": "Ambiguous: 'serves 2-4'" },
  "images": { "status": "not_found", "justification": "No images found" },
  "nutrition": { "status": "not_found", "justification": "Not provided" },
  "source": "microdata"
}
```

### Extraction Error

```json
{
  "error_type": "no_recipe_found",
  "message": "Page does not contain recipe content",
  "html_preview": "<html><head><title>Contact Us</title>..."
}
```
