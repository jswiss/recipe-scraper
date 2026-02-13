# Implementation Plan: Test Coverage

**Branch**: `007-test-coverage` | **Date**: 2026-02-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-test-coverage/spec.md`

## Summary

Add comprehensive automated tests aligned with specs 001-006. Fill coverage gaps in backup/restore and change log modules, add integration tests for the full pipeline (ingest → extract → tag → persist), ensure every spec acceptance scenario has a corresponding test, and verify Tauri command wrappers. All tests use fixture data and isolated databases — no network access required.

## Technical Context

**Language/Version**: Rust 1.77+ (stable toolchain)
**Primary Dependencies**: tauri 2.1.0, reqwest 0.12, rusqlite 0.38, serde 1.0, thiserror 2.x, robotstxt 0.3.0 (all existing — no new dependencies)
**Storage**: SQLite (existing database, WAL mode) via rusqlite with `bundled` feature
**Testing**: `cargo test` (built-in Rust test framework, `#[cfg(test)]` inline modules)
**Target Platform**: macOS (Tauri desktop application)
**Project Type**: Single Tauri application with Rust backend
**Performance Goals**: Full test suite completes in under 60 seconds
**Constraints**: No network access for tests (fixture data only), isolated temp databases per test
**Scale/Scope**: 163 existing tests across 20 files; adding ~80-100 new tests across 5 modules + integration tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | PASS | Tests will use descriptive names, one assertion focus per test, clear arrange-act-assert structure |
| II. AHA Programming | PASS | Test helpers will be created only when shared across 3+ tests; some duplication is acceptable |
| III. Minimal Dependencies | PASS | No new dependencies — uses built-in `#[cfg(test)]` and existing crate test features |
| IV. Accessibility First | N/A | No UI component in this feature |
| V. Monorepo + Open Source | PASS | Tests added within existing monorepo structure |
| VI. Local First | PASS | All tests use local fixture data and temp databases; no network or cloud dependencies |

## Project Structure

### Documentation (this feature)

```text
specs/007-test-coverage/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (test fixture schema)
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (test coverage matrix)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/src/
├── storage/
│   ├── backup.rs            # Add #[cfg(test)] module (currently untested)
│   └── change_log.rs        # Add #[cfg(test)] module (currently untested)
├── url_ingestion/
│   └── commands.rs          # Add #[cfg(test)] module for robots gate integration
├── robots_compliance/
│   └── commands.rs          # Add #[cfg(test)] module for command wrapper
├── recipe_extraction/
│   ├── json_ld.rs           # Add missing edge case tests
│   └── microdata.rs         # Add missing edge case tests
├── recipe_tagging/
│   └── commands.rs          # Add missing boundary tests
└── lib.rs                   # (no changes — integration tests go in tests/)

src-tauri/tests/
└── integration/
    ├── mod.rs               # Integration test module
    └── pipeline_test.rs     # End-to-end pipeline tests with fixture data

src-tauri/tests/fixtures/
├── jsonld_recipe.html       # Valid JSON-LD recipe page
├── microdata_recipe.html    # Valid Microdata recipe page
├── no_recipe.html           # Page without recipe markup
├── malformed_jsonld.html    # Invalid JSON-LD structure
├── robots_allow.txt         # robots.txt allowing RecipeScraper
├── robots_disallow.txt      # robots.txt disallowing RecipeScraper
└── robots_crawl_delay.txt   # robots.txt with Crawl-delay directive
```

**Structure Decision**: Tests follow the existing pattern of inline `#[cfg(test)]` modules for unit tests. Integration tests use Rust's standard `tests/` directory convention. Fixture files are stored in `tests/fixtures/` for shared access across integration tests.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
