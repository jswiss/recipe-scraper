# Data Model: Recipe Tagging and Categorization

**Feature**: 004-recipe-tagging | **Date**: 2026-02-07

## Entities

### Tag

A single categorization label with confidence score.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    /// The tag label (e.g., "Italian", "breakfast", "vegan")
    pub label: String,
    /// Confidence score between 0.0 and 1.0 inclusive
    pub confidence: f64,
}
```

**Validation rules**:
- `label` must be non-empty and match a label from the domain's vocabulary
- `confidence` must be in range [0.0, 1.0]
- Only tags with `confidence >= 0.5` are included in output (FR-006)

### TagDomain

Enum of the three fixed categorization domains.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagDomain {
    Cuisine,
    Course,
    Diet,
}
```

**Rules**: Fixed set of three values. Not extensible (custom tags are out of scope).

### TagSet

The complete tagging result for a recipe, organized by domain.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagSet {
    /// Cuisine tags, ordered by confidence (highest first)
    pub cuisine: Vec<Tag>,
    /// Course tags, ordered by confidence (highest first)
    pub course: Vec<Tag>,
    /// Diet tags, ordered by confidence (highest first)
    pub diet: Vec<Tag>,
}
```

**Invariants**:
- All three fields are always present (may be empty `Vec`)
- Tags within each field are sorted by `confidence` descending (FR-007)
- All tags have `confidence >= 0.5` (FR-006)
- Tag labels are drawn from the predefined vocabulary for that domain

### TaggingResult

Combined result for the `extract_and_tag` command.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaggingResult {
    /// The extracted recipe
    pub recipe: ExtractedRecipe,
    /// The assigned tags
    pub tags: TagSet,
}
```

### TaggingError

Error type for tagging operations.

```rust
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum TaggingError {
    /// Recipe has no usable content for tagging
    #[error("No taggable content: {message}")]
    NoContent { message: String },

    /// Extraction failed (only for extract_and_tag command)
    #[error("Extraction failed: {message}")]
    ExtractionFailed { message: String },
}
```

**Design note**: `NoContent` is a soft error — the system returns an empty `TagSet` rather than failing (FR-014). This error is reserved for cases where the function is called with a truly empty recipe. In practice, `tag_recipe` will return `Ok(TagSet::empty())` for sparse recipes and only error on invalid input.

## Vocabulary Data Structures

### CuisineEntry

Maps a cuisine label to its indicator keywords.

```rust
pub struct CuisineEntry {
    pub label: &'static str,
    /// Keywords that indicate this cuisine when found in recipe title
    pub title_keywords: &'static [&'static str],
    /// Ingredient names that indicate this cuisine
    pub ingredient_keywords: &'static [&'static str],
    /// Instruction/technique keywords that indicate this cuisine
    pub instruction_keywords: &'static [&'static str],
}
```

### CourseEntry

Maps a course label to its indicator keywords.

```rust
pub struct CourseEntry {
    pub label: &'static str,
    /// Keywords in title/description suggesting this course
    pub title_keywords: &'static [&'static str],
    /// Ingredient patterns suggesting this course
    pub ingredient_keywords: &'static [&'static str],
    /// Contextual cues (time of day, serving style)
    pub contextual_keywords: &'static [&'static str],
}
```

### DietDefinition

Defines a dietary category by exclusion rules.

```rust
pub struct DietDefinition {
    pub label: &'static str,
    /// Canonical ingredient names that violate this diet
    pub excluded_ingredients: &'static [&'static str],
    /// Ingredient categories that violate this diet (e.g., "meat", "dairy")
    pub excluded_categories: &'static [&'static str],
}
```

### IngredientAlias

Entry in the ingredient synonym map.

```rust
// Represented as HashMap<&'static str, &'static str>
// Key: normalized alias (e.g., "ap flour")
// Value: canonical name (e.g., "wheat flour")
```

### IngredientProperties

Maps canonical ingredient names to dietary properties.

```rust
// Represented as HashMap<&'static str, &'static [DietaryFlag]>

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DietaryFlag {
    ContainsMeat,
    ContainsPoultry,
    ContainsFish,
    ContainsDairy,
    ContainsEggs,
    ContainsGluten,
    ContainsNuts,
    ContainsSoy,
    HighCarb,
    HighFat,
    ContainsSugar,
}
```

## Relationships

```
ExtractedRecipe (input, from recipe_extraction module)
        │
        ▼
  tag_recipe_from_extracted()
        │
        ├──► cuisine_tagger::tag() ──► Vec<Tag>
        ├──► course_tagger::tag()  ──► Vec<Tag>
        └──► diet_tagger::tag()    ──► Vec<Tag>
                                          │
                                          ▼
                                       TagSet
                                    (cuisine, course, diet)
```

## Serialization Format (JSON)

```json
{
  "cuisine": [
    { "label": "Thai", "confidence": 0.85 },
    { "label": "Southeast Asian", "confidence": 0.60 }
  ],
  "course": [
    { "label": "dinner", "confidence": 0.75 },
    { "label": "main course", "confidence": 0.70 }
  ],
  "diet": [
    { "label": "gluten-free", "confidence": 0.90 },
    { "label": "dairy-free", "confidence": 0.85 }
  ]
}
```

## State Transitions

Tagging is stateless — no lifecycle. Input recipe in, tag set out. No mutations, no side effects.
