# Tauri Command Contracts: Recipe Tagging

**Feature**: 004-recipe-tagging | **Date**: 2026-02-07

## Commands

### `tag_recipe`

Tag an already-extracted recipe. Supports optional heuristic refinement mode.

**Signature**:
```rust
#[tauri::command]
pub async fn tag_recipe(
    recipe: ExtractedRecipe,
    refine: Option<bool>,
) -> Result<TagSet, TaggingError>
```

**Input (from frontend)**:
```typescript
const tags = await invoke<TagSet>('tag_recipe', {
  recipe: extractedRecipe,  // ExtractedRecipe object
  refine: false,            // Optional: enable heuristic refinement
});
```

**Input JSON**:
```json
{
  "recipe": { /* ExtractedRecipe object */ },
  "refine": false
}
```

**Success Response** (`TagSet`):
```json
{
  "cuisine": [
    { "label": "Thai", "confidence": 0.85 },
    { "label": "Southeast Asian", "confidence": 0.60 }
  ],
  "course": [
    { "label": "dinner", "confidence": 0.75 }
  ],
  "diet": [
    { "label": "gluten-free", "confidence": 0.90 }
  ]
}
```

**Error Response** (`TaggingError`):
```json
{
  "error_type": "no_content",
  "message": "Recipe has no title, ingredients, or instructions"
}
```

**Behavior**:
- `refine` defaults to `false` if omitted
- When `refine = false`: Uses rule-based keyword matching (fast, default)
- When `refine = true`: Uses heuristic scoring with co-occurrence analysis (more accurate)
- Returns empty `Vec` for any domain where no tags meet the 0.3 threshold
- Tags within each domain are sorted by confidence descending

---

### `extract_and_tag`

Extract a recipe from HTML and automatically tag it. Satisfies FR-018 (auto-tag after extraction).

**Signature**:
```rust
#[tauri::command]
pub async fn extract_and_tag(
    html: String,
) -> Result<TaggingResult, TaggingError>
```

**Input (from frontend)**:
```typescript
const result = await invoke<TaggingResult>('extract_and_tag', {
  html: htmlContent,
});
// result.recipe - the extracted recipe
// result.tags   - the assigned tags
```

**Success Response** (`TaggingResult`):
```json
{
  "recipe": {
    "title": { "status": "found", "value": "Pad Thai" },
    "ingredients": [
      { "name": "rice noodles", "quantity": 8.0, "unit": "oz", "raw_text": "8 oz rice noodles" },
      { "name": "fish sauce", "quantity": 3.0, "unit": "tbsp", "raw_text": "3 tbsp fish sauce" }
    ],
    "...": "..."
  },
  "tags": {
    "cuisine": [{ "label": "Thai", "confidence": 0.85 }],
    "course": [{ "label": "dinner", "confidence": 0.70 }],
    "diet": [{ "label": "gluten-free", "confidence": 0.80 }]
  }
}
```

**Error Response** (`TaggingError`):
```json
{
  "error_type": "extraction_failed",
  "message": "No structured recipe data found (JSON-LD or Microdata)"
}
```

**Behavior**:
- Internally calls `extract_recipe_internal` then `tag_recipe_from_extracted`
- Always uses rule-based tagging (default mode) for auto-tag
- Maps `ExtractionError` to `TaggingError::ExtractionFailed`
- If extraction succeeds but tagging finds no content, returns recipe with empty tag sets

---

## Type Definitions (TypeScript equivalents for frontend)

```typescript
interface Tag {
  label: string;
  confidence: number;  // 0.0 to 1.0
}

interface TagSet {
  cuisine: Tag[];
  course: Tag[];
  diet: Tag[];
}

interface TaggingResult {
  recipe: ExtractedRecipe;
  tags: TagSet;
}

interface TaggingError {
  error_type: 'no_content' | 'extraction_failed';
  message: string;
}
```
