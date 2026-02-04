# Implementation Plan: Recipe Structure Extraction

**Branch**: `003-recipe-extraction` | **Date**: 2026-02-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-recipe-extraction/spec.md`

## Summary

Extract structured recipe data from HTML content using a priority chain: JSON-LD schema → Microdata → Local AI fallback (Gemma-2-2B). All fields are returned with values or explicit null justifications, following the schema.org Recipe vocabulary.

## Technical Context

**Language/Version**: Rust 1.77+ (stable toolchain)
**Primary Dependencies**:
- `scraper` 0.17+ (HTML parsing, CSS selectors)
- `serde_json` 1.0 (JSON-LD parsing, already in Cargo.toml)
- `llama-cpp-sys-3` 0.5+ (local LLM inference)

**Storage**: N/A (extraction only; storage is separate feature)
**Testing**: `cargo test` (unit + integration tests)
**Target Platform**: macOS (Darwin), Linux, Windows via Tauri
**Project Type**: Single (Rust backend in Tauri)
**Performance Goals**:
- JSON-LD/Microdata extraction: <100ms
- AI fallback: <5 seconds
- 95% accuracy on structured data, 80% on AI fallback

**Constraints**:
- Offline-capable (local-first principle)
- AI model size <5GB
- No cloud services for extraction

**Scale/Scope**: Single recipe per extraction request

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | PASS | CSS selectors are declarative; extraction logic is straightforward |
| II. AHA Programming | PASS | No abstractions planned; direct extraction from JSON-LD/Microdata |
| III. Minimal Dependencies | PASS | Only 2 new crates: `scraper` (battle-tested), `llama-cpp-sys-3` (minimal FFI) |
| IV. Accessibility First | N/A | Backend-only feature |
| V. Monorepo + Open Source | PASS | All dependencies are open source; no cloud lock-in |
| VI. Local First | PASS | Local AI model; works fully offline |

### Post-Design Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | PASS | Data model uses clear types with explicit null handling |
| II. AHA Programming | PASS | Three extractors (JSON-LD, Microdata, AI) share interface but not premature abstraction |
| III. Minimal Dependencies | PASS | `scraper` adds ~6 transitive deps; `llama-cpp-sys-3` adds ~5-7; total is reasonable |
| IV. Accessibility First | N/A | Backend-only feature |
| V. Monorepo + Open Source | PASS | Model from HuggingFace (open weights); all code in monorepo |
| VI. Local First | PASS | Model downloaded to local config dir; no network required after setup |

## Project Structure

### Documentation (this feature)

```text
specs/003-recipe-extraction/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0: Technology research
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Developer quickstart
├── contracts/           # Phase 1: API contracts
│   └── tauri-commands.md
└── tasks.md             # Phase 2: Implementation tasks (via /speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/src/
├── lib.rs                    # Update: register new commands and state
├── main.rs                   # Unchanged
├── url_ingestion/            # Existing module (unchanged)
│   ├── mod.rs
│   ├── models.rs
│   ├── validator.rs
│   ├── normalizer.rs
│   ├── fetcher.rs
│   └── commands.rs
└── recipe_extraction/        # New module
    ├── mod.rs                # Module exports
    ├── models.rs             # ExtractedRecipe, Ingredient, NutritionInfo, etc.
    ├── json_ld.rs            # JSON-LD extraction (priority 1)
    ├── microdata.rs          # Microdata extraction (priority 2)
    ├── ai_fallback.rs        # Local LLM inference (priority 3)
    ├── duration.rs           # ISO 8601 duration parsing
    └── commands.rs           # Tauri commands: extract_recipe, etc.
```

**Structure Decision**: Single module `recipe_extraction` added to existing Tauri backend, following the pattern established by `url_ingestion`.

## Complexity Tracking

> No constitution violations identified. Design follows all principles.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | - | - |

## Phase Summary

### Phase 0: Research (Complete)

- [x] JSON-LD parsing approach: `serde_json` (no full JSON-LD library needed)
- [x] Microdata parsing approach: `scraper` with CSS selectors
- [x] Local AI approach: `llama-cpp-sys-3` with Gemma-2-2B Q4_K_M
- [x] Schema.org Recipe field mapping documented

**Output**: [research.md](./research.md)

### Phase 1: Design (Complete)

- [x] Data model: `ExtractedRecipe`, `ExtractedField<T>`, `Ingredient`, etc.
- [x] API contracts: Tauri commands for extraction
- [x] Project structure: `recipe_extraction` module layout
- [x] Constitution check: All principles satisfied

**Outputs**:
- [data-model.md](./data-model.md)
- [contracts/tauri-commands.md](./contracts/tauri-commands.md)
- [quickstart.md](./quickstart.md)

### Phase 2: Tasks (Pending)

Run `/speckit.tasks` to generate implementation tasks.

## Dependencies

### External Crates (to add)

```toml
# HTML parsing (battle-tested, Servo-based)
scraper = "0.17"

# Local LLM inference (minimal FFI bindings)
llama-cpp-sys-3 = { version = "0.5", features = ["native"] }
```

### Existing Crates (no changes)

- `serde`, `serde_json`: Serialization (already present)
- `reqwest`: HTTP client for model download (already present)
- `tokio`: Async runtime (already present)
- `thiserror`: Error handling (already present)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AI model too large | Low | Medium | Gemma-2-2B Q4 is 1.5-1.8GB, well under 5GB limit |
| llama-cpp compilation issues | Medium | Medium | Use pre-built binaries if available; document build requirements |
| Poor extraction accuracy | Medium | Medium | Prioritize structured data; AI is last resort |
| Slow AI inference | Low | Low | Async execution prevents UI blocking; 2-5s acceptable for fallback |
