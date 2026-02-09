# Implementation Plan: Recipe Tagging and Categorization

**Branch**: `004-recipe-tagging` | **Date**: 2026-02-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-recipe-tagging/spec.md`

## Summary

Add a `recipe_tagging` module that assigns cuisine, course, and diet tags with confidence scores to an `ExtractedRecipe`. Default mode uses rule-based keyword/pattern matching with predefined vocabularies (~30-50 labels per domain) and a normalized ingredient synonym map for diet safety. An optional heuristic refinement mode uses weighted multi-signal scoring for improved cuisine/course accuracy. Tagging auto-runs after extraction and is also callable on-demand via a separate Tauri command.

## Technical Context

**Language/Version**: Rust 1.77+ (stable toolchain)
**Primary Dependencies**: serde 1.x, thiserror 2.x, tauri 2.x (all already in Cargo.toml; no new dependencies)
**Storage**: N/A (tagging is compute-only; persistence is a separate feature)
**Testing**: `cargo test` with inline `#[cfg(test)]` modules following existing patterns
**Target Platform**: Desktop (macOS, Windows, Linux via Tauri)
**Project Type**: Single (Tauri desktop app — `src-tauri/`)
**Performance Goals**: <1 second per recipe tagging (SC-001)
**Constraints**: Local-first, no network access, fully offline, no external ML models
**Scale/Scope**: Single recipe at a time; ~35 cuisine labels, ~12 course labels, ~15 diet labels

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | PASS | Follows established module pattern (mod.rs/models.rs/commands.rs). Single-purpose functions for each tagging domain. |
| II. AHA Programming | PASS | Three domain taggers share a common `Tag` type but have separate logic — no premature abstraction of tagging strategy. Vocabulary data is structured similarly but not forced into a shared framework. |
| III. Minimal Dependencies | PASS | Zero new dependencies. Rule-based matching uses only `std` string operations + existing `serde`/`thiserror`. |
| IV. Accessibility First | N/A | Backend-only module; no UI components. |
| V. Monorepo + Open Source | PASS | New module in existing `src-tauri/src/` directory. |
| VI. Local First | PASS | All tagging is local computation on recipe data fields. No network, no external services. |

**Gate result**: PASS — no violations.

## Project Structure

### Documentation (this feature)

```text
specs/004-recipe-tagging/
├── plan.md              # This file
├── research.md          # Phase 0: Research decisions
├── data-model.md        # Phase 1: Data model design
├── quickstart.md        # Phase 1: Developer quickstart
├── contracts/           # Phase 1: Tauri command contracts
│   └── tauri-commands.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/src/
├── lib.rs                       # Add recipe_tagging module + commands
├── recipe_extraction/           # Existing (input source)
│   ├── models.rs                # ExtractedRecipe, Ingredient, etc.
│   └── commands.rs              # extract_recipe command
└── recipe_tagging/              # NEW MODULE
    ├── mod.rs                   # Module exports
    ├── models.rs                # Tag, TagDomain, TagSet, TaggingError
    ├── commands.rs              # Tauri commands: tag_recipe, extract_and_tag
    ├── vocabulary.rs            # Predefined tag labels + keyword maps per domain
    ├── cuisine_tagger.rs        # Cuisine tagging logic
    ├── course_tagger.rs         # Course tagging logic
    ├── diet_tagger.rs           # Diet tagging logic + ingredient synonym map
    └── scoring.rs               # Confidence score computation + optional heuristic refinement
```

**Structure Decision**: Follows the established module pattern from `url_ingestion/` and `recipe_extraction/`. Each tagging domain gets its own file to keep functions focused and testable. Shared types live in `models.rs`, shared vocabulary data in `vocabulary.rs`, and scoring logic in `scoring.rs`.

## Complexity Tracking

> No constitution violations — table not needed.

