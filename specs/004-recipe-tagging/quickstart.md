# Quickstart: Recipe Tagging and Categorization

**Feature**: 004-recipe-tagging | **Date**: 2026-02-07

## Prerequisites

- Rust 1.77+ (stable toolchain)
- Existing `src-tauri/` project builds successfully (`cargo build`)

## Build & Test

```bash
cd src-tauri
cargo build          # Build with new recipe_tagging module
cargo test           # Run all tests including tagging
cargo test recipe_tagging  # Run only tagging tests
cargo clippy         # Lint check
```

## Module Location

```
src-tauri/src/recipe_tagging/
├── mod.rs              # Public API
├── models.rs           # Tag, TagSet, TaggingError
├── commands.rs         # Tauri commands
├── vocabulary.rs       # Tag labels + keyword maps
├── cuisine_tagger.rs   # Cuisine tagging logic
├── course_tagger.rs    # Course tagging logic
├── diet_tagger.rs      # Diet tagging + ingredient synonyms
└── scoring.rs          # Confidence computation
```

## Usage (Rust library)

```rust
use recipe_extraction::ExtractedRecipe;
use recipe_tagging::{tag_recipe_from_extracted, TagSet};

let recipe: ExtractedRecipe = /* ... */;

// Default rule-based tagging
let tags: TagSet = tag_recipe_from_extracted(&recipe, false);

// With heuristic refinement
let refined_tags: TagSet = tag_recipe_from_extracted(&recipe, true);
```

## Usage (Frontend via Tauri IPC)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Option 1: Extract and auto-tag in one step
const result = await invoke<TaggingResult>('extract_and_tag', { html });
console.log(result.tags.cuisine);  // [{ label: "Thai", confidence: 0.85 }]

// Option 2: Tag an already-extracted recipe
const tags = await invoke<TagSet>('tag_recipe', {
  recipe: extractedRecipe,
  refine: false,
});

// Option 3: Re-tag with heuristic refinement
const refinedTags = await invoke<TagSet>('tag_recipe', {
  recipe: extractedRecipe,
  refine: true,
});
```

## Key Design Decisions

1. **No new dependencies** — rule-based matching uses only `std` string operations
2. **Stateless** — no shared mutable state; each call is independent
3. **Fail-safe diet tagging** — ambiguous ingredients reduce confidence; missing ingredients yield no diet tags
4. **Separate from extraction** — tagging module imports `ExtractedRecipe` but doesn't modify the extraction module
