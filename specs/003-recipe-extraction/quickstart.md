# Quickstart: Recipe Structure Extraction

**Feature**: 003-recipe-extraction
**Date**: 2026-02-04

## Prerequisites

- Rust 1.77+ (stable toolchain)
- Tauri CLI (`cargo install tauri-cli`)
- Existing URL ingestion module (from 001-url-ingestion)

## New Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
# HTML parsing
scraper = "0.17"

# Local LLM inference (for AI fallback)
llama-cpp-sys-3 = { version = "0.5", features = ["native"] }
```

## Module Structure

```
src-tauri/src/
├── lib.rs                    # Update to register new commands
├── url_ingestion/            # Existing module
└── recipe_extraction/        # New module
    ├── mod.rs                # Module exports
    ├── models.rs             # ExtractedRecipe, Ingredient, etc.
    ├── json_ld.rs            # JSON-LD extraction
    ├── microdata.rs          # Microdata extraction
    ├── ai_fallback.rs        # Local LLM inference
    ├── duration.rs           # ISO 8601 duration parsing
    └── commands.rs           # Tauri commands
```

## Quick Test

After implementation, test extraction:

```bash
cd src-tauri
cargo test recipe_extraction

# Integration test with actual URL
cargo test --test integration extract_from_url
```

## Frontend Usage

```typescript
import { invoke } from '@tauri-apps/api/core';

// Extract from HTML (if you already fetched)
const recipe = await invoke('extract_recipe', { html: htmlContent });

// Extract from URL (convenience)
const recipe = await invoke('extract_recipe_from_url', { url: 'https://example.com/recipe' });

// Check AI model status
const status = await invoke('check_ai_model_status');
if (!status.downloaded) {
  await invoke('download_ai_model');
}
```

## Key Files to Create

1. `src-tauri/src/recipe_extraction/mod.rs` - Module exports
2. `src-tauri/src/recipe_extraction/models.rs` - Data types
3. `src-tauri/src/recipe_extraction/json_ld.rs` - JSON-LD extractor
4. `src-tauri/src/recipe_extraction/microdata.rs` - Microdata extractor
5. `src-tauri/src/recipe_extraction/ai_fallback.rs` - LLM inference
6. `src-tauri/src/recipe_extraction/commands.rs` - Tauri command handlers

## Extraction Priority

1. **JSON-LD** (fastest, most reliable)
2. **Microdata** (fallback for older sites)
3. **AI Fallback** (last resort, requires model download)

## Performance Targets

- JSON-LD extraction: <10ms
- Microdata extraction: <50ms
- AI fallback: 2-5 seconds (first inference loads model)
