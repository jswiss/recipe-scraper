# Research: Recipe Structure Extraction

**Feature**: 003-recipe-extraction
**Date**: 2026-02-04
**Status**: Complete

## Executive Summary

This document captures research findings for implementing recipe structure extraction from HTML content. The feature requires extracting recipe data from JSON-LD, Microdata, and falling back to local AI inference when no structured data is present.

---

## 1. JSON-LD Parsing

### Decision: Use `serde_json` directly with `scraper` for HTML extraction

### Rationale
- `serde_json` is already in Cargo.toml for serialization
- Full JSON-LD processing libraries (`json-ld`, `sophia`) add 11+ transitive dependencies
- Recipe schemas are simple objects that don't require full JSON-LD expansion/compaction
- Aligns with Constitution Principle III (Minimal Dependencies)

### Alternatives Considered

| Option | Dependencies | Complexity | Decision |
|--------|-------------|------------|----------|
| `json-ld` crate | 11+ transitive | Full JSON-LD 1.1 spec | Rejected: Overkill for recipe extraction |
| `sophia` crate | 6+ transitive | RDF/Linked Data toolkit | Rejected: Designed for full RDF processing |
| `serde_json` | 0 (already present) | Simple JSON parsing | **Selected**: Sufficient for recipe schemas |

### Implementation Approach

1. Use `scraper` to find `<script type="application/ld+json">` tags
2. Parse each script content with `serde_json::from_str()`
3. Check for `@type: "Recipe"` (string or array)
4. Deserialize into typed Recipe struct

### Edge Cases

- **Graph mode**: `"@graph": [{@type: Recipe}, {...}]` - flatten and find first Recipe
- **Array @type**: `"@type": ["Recipe", "Thing"]` - check if "Recipe" in array
- **Multiple recipes**: Extract first/primary recipe per spec

---

## 2. Microdata Parsing

### Decision: Use `scraper` crate with CSS selectors for itemprop extraction

### Rationale
- No dedicated Microdata parser exists in Rust ecosystem
- `scraper` provides CSS selector-based DOM querying (wraps `html5ever`)
- Simple extraction pattern: find `[itemscope][itemtype*="schema.org/Recipe"]`, then extract `[itemprop]` children
- Battle-tested (used by Servo browser), 2.3k GitHub stars

### Alternatives Considered

| Option | Dependencies | Decision |
|--------|-------------|----------|
| `html5ever` (direct) | 3-4 | Rejected: Too low-level, requires manual tree building |
| `select` crate | Similar to scraper | Rejected: Deprecated, community uses scraper |
| `scraper` | 6 (html5ever, selectors, ego-tree) | **Selected**: High-level, readable API |

### Implementation Approach

1. Find recipe container: `[itemscope][itemtype*="schema.org/Recipe"]`
2. Extract properties via `[itemprop="name"]`, `[itemprop="recipeIngredient"]`, etc.
3. Handle `content` attribute (for dates/durations) vs text content
4. Support nested itemscope for complex ingredients/instructions

---

## 3. Local AI Fallback

### Decision: Use `llama-cpp-sys-3` with Gemma-2-2B (Q4_K_M quantization)

### Rationale
- llama.cpp is the most mature ecosystem for local LLM inference
- Minimal dependencies (C/C++ bindings, no complex build chains)
- GGUF format enables 4-bit quantization (1.5-3GB model size)
- Optimized for Apple Silicon (Darwin environment) and x86
- Aligns with Constitution Principle VI (Local First)

### Alternatives Considered

| Option | Dependencies | Binary Size | Decision |
|--------|-------------|-------------|----------|
| `llama-cpp-sys-3` | 5-7 | +30-50MB | **Selected**: Minimal, mature |
| `candle` | 40+ | +50-100MB | Rejected: Heavier, pure Rust |
| `mistral.rs` | 60+ | +80-150MB | Rejected: Maximum convenience but heavy |

### Model Selection

| Model | Size (Q4) | Size (Q8) | Performance |
|-------|-----------|-----------|-------------|
| **Gemma-2-2B** | 1.5-1.8GB | 2.8-3GB | 2-3s on M1/M2 Mac |
| Gemma-2-7B | 3.5-4.5GB | 7-8GB | Approaches 5GB budget |

**Recommendation**: Gemma-2-2B with Q4_K_M quantization (best quality-to-size ratio)

### Model Sources
- HuggingFace: `bartowski/gemma-2-2b-it-GGUF`
- HuggingFace: `MaziyarPanahi/gemma-2b-GGUF`

### Integration Strategy

1. **Model Download**: On first run, download to `~/.config/recipe-scraper/models/`
2. **Lazy Loading**: Load model once into Tauri state, reuse for all extractions
3. **Async Execution**: Use `tokio::task::spawn_blocking()` to avoid UI blocking
4. **Prompt Engineering**: Craft prompt for recipe field extraction from HTML

---

## 4. Schema.org Recipe Fields

### Standard Fields (schema.org/Recipe)

| Property | Type | Required | Notes |
|----------|------|----------|-------|
| `name` | Text | Yes | Recipe title |
| `description` | Text | No | Summary |
| `image` | URL/ImageObject | No | Can be array |
| `recipeIngredient` | Text[] | Yes | List of ingredients |
| `recipeInstructions` | Text[]/HowToStep[] | Yes | Ordered steps |
| `prepTime` | Duration (ISO 8601) | No | e.g., "PT15M" |
| `cookTime` | Duration (ISO 8601) | No | e.g., "PT30M" |
| `totalTime` | Duration (ISO 8601) | No | prepTime + cookTime |
| `recipeYield` | Text/QuantitativeValue | No | e.g., "4 servings" |
| `nutrition` | NutritionInformation | No | Nested object |

### Duration Parsing (ISO 8601)

Format: `PT[n]H[n]M[n]S`
- `PT15M` = 15 minutes
- `PT1H30M` = 90 minutes
- `PT2H` = 120 minutes

**Implementation**: Manual parsing (~30 lines) or `iso8601` crate (2KB, no deps)

### NutritionInformation Fields

| Property | Type | Unit |
|----------|------|------|
| `calories` | Energy | kcal |
| `fatContent` | Mass | grams |
| `saturatedFatContent` | Mass | grams |
| `carbohydrateContent` | Mass | grams |
| `sugarContent` | Mass | grams |
| `fiberContent` | Mass | grams |
| `proteinContent` | Mass | grams |
| `sodiumContent` | Mass | mg |

---

## 5. Dependency Summary

### New Dependencies Required

| Crate | Version | Purpose | Transitive Deps |
|-------|---------|---------|-----------------|
| `scraper` | 0.17+ | HTML parsing, CSS selectors | ~6 |
| `llama-cpp-sys-3` | 0.5+ | Local LLM inference | ~5-7 |

### Existing Dependencies (Already in Cargo.toml)

- `serde`, `serde_json`: JSON parsing
- `reqwest`: HTTP (for model download)
- `tokio`: Async runtime
- `thiserror`: Error handling

### Constitution Compliance

- **Principle I (Readable)**: CSS selectors are declarative and self-documenting
- **Principle II (AHA)**: No premature abstractions; extract JSON-LD/Microdata directly
- **Principle III (Minimal Deps)**: Only 2 new crates, both well-maintained
- **Principle VI (Local First)**: Local AI model, no cloud services required

---

## 6. Performance Considerations

### Extraction Priority (Fast Path First)

1. JSON-LD extraction: <10ms (JSON parse only)
2. Microdata extraction: <50ms (DOM traversal)
3. Local AI fallback: 2-5 seconds (model inference)

### Memory Budget

| Component | Memory |
|-----------|--------|
| Loaded model (Q4) | 2-3GB |
| Inference buffers | ~500MB |
| HTML document | <10MB |

### Caching Strategy

- Load model once on first fallback use
- Keep model in memory until app shutdown
- No caching of extracted recipes (handled by storage layer)

---

## 7. Open Questions Resolved

| Question | Resolution |
|----------|------------|
| Which HTML parser? | `scraper` (wraps html5ever, CSS selector API) |
| Full JSON-LD library? | No, `serde_json` sufficient for recipe schemas |
| Which local LLM crate? | `llama-cpp-sys-3` (minimal deps, mature) |
| Which model? | Gemma-2-2B Q4_K_M (1.5-1.8GB) |
| Model bundling? | Download on first use, store in config dir |
