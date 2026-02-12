# Implementation Plan: Robots.txt Compliance

**Branch**: `006-robots-txt-compliance` | **Date**: 2026-02-11 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-robots-txt-compliance/spec.md`

## Summary

Add robots.txt compliance checking to the recipe scraper. Before any URL is scraped, the system fetches and parses the site's robots.txt (RFC 9309), returning a decision object indicating whether scraping is allowed/disallowed and any crawl delay. The compliance check is exposed as a standalone Tauri command and also enforced as a gate within the existing `ingest_url` flow. Parsed robots.txt files are cached in SQLite with a 24-hour TTL for offline support.

Uses the `robotstxt` crate (zero-dependency port of Google's production parser) for RFC 9309 compliance, plus a lightweight custom `Crawl-delay` extractor.

## Technical Context

**Language/Version**: Rust 1.77+ (stable toolchain)
**Primary Dependencies**: tauri 2.1.0, reqwest 0.12, rusqlite 0.38, serde 1.0, thiserror 2.x, robotstxt 0.3.0 (NEW — zero transitive deps)
**Storage**: SQLite (existing database, new migration 002 for `robots_cache` table)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Desktop (macOS, Linux, Windows) via Tauri
**Project Type**: Single (Tauri desktop app)
**Performance Goals**: <2s latency for first compliance check per domain; <10ms for cached checks
**Constraints**: Offline-capable (cached decisions), max 500KB robots.txt, 10s fetch timeout
**Scale/Scope**: Single-user desktop app, in-memory + SQLite cache per domain

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | PASS | Separate module with single-purpose functions (fetch, parse, cache, decide) |
| II. AHA Programming | PASS | Using proven `robotstxt` crate instead of premature custom abstraction; custom Crawl-delay parser is <30 lines |
| III. Minimal Dependencies | PASS | `robotstxt` has zero transitive dependencies; only 1 new crate added |
| IV. Accessibility First | N/A | Backend-only feature, no UI |
| V. Monorepo + Open Source | PASS | All code in single repo; `robotstxt` is Apache 2.0 |
| VI. Local First | PASS | SQLite cache with 24h TTL enables offline compliance checks; no cloud service required |

**Post-Phase 1 Re-check**: All gates still pass. No violations detected.

## Project Structure

### Documentation (this feature)

```text
specs/006-robots-txt-compliance/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── tauri-commands.md # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/src/
├── robots_compliance/           # NEW module
│   ├── mod.rs                   # Module exports
│   ├── models.rs                # RobotsDecision, RobotsError, CacheSource
│   ├── checker.rs               # Core logic: fetch, parse, cache, decide
│   ├── crawl_delay.rs           # Crawl-delay directive parser
│   └── commands.rs              # Tauri command wrapper
├── storage/
│   └── migrations/
│       └── 002_robots_cache.sql # NEW migration
├── url_ingestion/
│   ├── models.rs                # UPDATED: add RobotsDisallowed variant
│   └── commands.rs              # UPDATED: call compliance gate
└── lib.rs                       # UPDATED: register new command
```

**Structure Decision**: Follows the established module-per-feature pattern (url_ingestion, recipe_extraction, recipe_tagging, storage). The new `robots_compliance` module is a peer module with its own models, logic, and Tauri command.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
