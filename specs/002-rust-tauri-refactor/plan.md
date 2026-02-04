# Implementation Plan: Rust/Tauri Backend Refactor

**Branch**: `002-rust-tauri-refactor` | **Date**: 2026-02-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-rust-tauri-refactor/spec.md`

## Summary

Refactor the Python URL ingestion module to Rust, exposing functionality as async Tauri v2 commands. The Rust implementation provides identical functionality (validate, normalize, fetch URLs) with structured error handling. After verification, all Python code will be removed.

## Technical Context

**Language/Version**: Rust 1.70+ (stable toolchain)
**Primary Dependencies**: tauri 2.x, reqwest 0.12+, url 2.x, idna 1.x, serde 1.x, thiserror 2.x
**Storage**: N/A (stateless URL fetching)
**Testing**: cargo test (unit + integration)
**Target Platform**: Desktop (macOS, Windows, Linux) via Tauri
**Project Type**: Single project (Tauri app with Rust backend)
**Performance Goals**: URL validation <10ms, fetch completion follows 30s timeout
**Constraints**: Offline-capable for validation, 10MB response limit, 5 redirect max
**Scale/Scope**: Single-user desktop application

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | ✅ PASS | Rust module structure mirrors Python for familiarity; small focused functions |
| II. AHA Programming | ✅ PASS | Direct port of proven Python patterns; no new abstractions |
| III. Minimal Dependencies | ✅ PASS | All deps are standard Rust ecosystem choices (reqwest, serde, url); no trivial deps |
| IV. Accessibility First | N/A | Backend-only; no UI in this feature |
| V. Monorepo + Open Source | ✅ PASS | Single repo; Tauri is open source, self-hostable |
| VI. Local First | ✅ PASS | URL validation works offline; no cloud services required |

**Gate Result**: PASSED - Proceed to implementation

## Project Structure

### Documentation (this feature)

```text
specs/002-rust-tauri-refactor/
├── spec.md              # Feature requirements
├── plan.md              # This file
├── research.md          # Technology decisions
├── data-model.md        # Entity definitions
├── quickstart.md        # Usage examples
├── contracts/           # TypeScript type definitions
│   └── tauri-commands.ts
└── tasks.md             # Implementation tasks (via /speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json         # Tauri v2 configuration
├── build.rs                # Tauri build script
├── icons/                  # App icons
└── src/
    ├── main.rs             # Tauri app entry point, command registration
    └── url_ingestion/      # URL ingestion module
        ├── mod.rs          # Module exports
        ├── models.rs       # NormalizedUrl, FetchSuccess, FetchError, ErrorType
        ├── validator.rs    # URL syntax/protocol validation
        ├── normalizer.rs   # URL normalization (IDN, ports, case)
        ├── fetcher.rs      # HTTP fetching with reqwest
        └── commands.rs     # Tauri #[command] wrappers

# Python files to be removed after Rust verification:
src/url_ingestion/          # DELETE after Rust works
tests/                      # DELETE after Rust works
pyproject.toml              # DELETE after Rust works
```

**Structure Decision**: Standard Tauri v2 layout with `src-tauri/` directory. The URL ingestion module is organized as a Rust submodule with the same file structure as Python for easy comparison during implementation.

## Complexity Tracking

> No constitution violations requiring justification.

## Phase Outputs

- [x] research.md - Technology decisions documented
- [x] data-model.md - Entity definitions complete
- [x] contracts/tauri-commands.ts - TypeScript types for frontend
- [x] quickstart.md - Usage examples
- [x] tasks.md - Implementation task breakdown (49 tasks across 8 phases)
