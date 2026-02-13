# Data Model: Test Coverage

This feature adds no new persistent data entities. This document defines the **test fixture data** schema — the known inputs and expected outputs used to validate each module.

## Test Fixture: JSON-LD Recipe HTML

Represents a complete recipe page with valid JSON-LD structured data.

| Field | Value | Purpose |
|-------|-------|---------|
| Title | "Classic Chocolate Chip Cookies" | Known title for assertion |
| Description | "Crispy edges, chewy centers..." | Known description |
| Ingredients | 6 items (flour, sugar, butter, eggs, vanilla, chocolate chips) | Covers structured parsing |
| Instructions | 4 steps | Covers HowToStep parsing |
| Prep time | PT15M (15 minutes) | ISO 8601 duration |
| Cook time | PT12M (12 minutes) | ISO 8601 duration |
| Servings | "24 cookies" | String serving size |
| Images | 1 URL | Image extraction |
| Nutrition | calories: 220, fat: 11g, carbs: 28g, protein: 3g | Full nutrition block |

**Expected tags** (for tagging verification):
- Cuisine: [] (no strong cuisine signal)
- Course: ["dessert" (>0.7), "snack" (>0.5)]
- Diet: [] (contains butter, eggs, flour — excludes vegan, vegetarian, gluten-free, dairy-free)

## Test Fixture: Microdata Recipe HTML

Same recipe content encoded as Microdata `itemscope itemtype="https://schema.org/Recipe"`.

| Field | Value | Purpose |
|-------|-------|---------|
| Same fields as JSON-LD fixture | Same values | Validates Microdata produces identical output |

## Test Fixture: No Recipe HTML

A regular web page without any recipe markup.

| Content | Purpose |
|---------|---------|
| Basic HTML with navigation, article text, footer | Validates ExtractionError::NoRecipeFound |

## Test Fixture: Malformed JSON-LD HTML

Page with JSON-LD that has structural errors.

| Content | Purpose |
|---------|---------|
| `<script type="application/ld+json">{ "broken": }` | Validates graceful error handling |

## Test Fixture: robots.txt Variants

| Fixture | Content | Expected Decision |
|---------|---------|-------------------|
| robots_allow.txt | `User-agent: *\nAllow: /` | allowed=true |
| robots_disallow.txt | `User-agent: RecipeScraper\nDisallow: /` | allowed=false |
| robots_crawl_delay.txt | `User-agent: *\nCrawl-delay: 5\nAllow: /` | allowed=true, crawl_delay=5.0 |

## Test Fixture: Backup Database

Not a static file — constructed programmatically in test setup.

| Content | Purpose |
|---------|---------|
| In-memory DB with 3 recipes, tags, ingredients | Roundtrip integrity verification |
| Empty file (0 bytes) | Corrupted backup error handling |
| File without `recipes` table | Invalid backup error handling |

## Relationships

```
JSON-LD HTML fixture
  → extract_recipe() → ExtractedRecipe
    → tag_recipe() → TagSet
      → save_recipe(db) → SavedRecipe
        → backup_collection_to(file) → BackupResult
          → restore_collection_from(file) → RestoredRecipe (matches original)
```

This chain validates the full pipeline roundtrip from fixture data through persistence and backup.
