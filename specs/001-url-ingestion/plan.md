# Implementation Plan: URL Ingestion

**Branch**: `001-url-ingestion` | **Date**: 2026-02-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-url-ingestion/spec.md`

## Summary

Implement a URL ingestion module that accepts recipe URLs, validates and normalizes them per
RFC 3986, fetches the HTML content, and returns a structured result (success with HTML or
failure with typed error). The module will use Python's standard library for URL parsing and
validation, with `requests` for HTTP fetching.

## Technical Context

**Language/Version**: Python 3.11+
**Primary Dependencies**: requests (HTTP client), idna (Punycode/IDN support)
**Storage**: N/A (stateless module, no persistence)
**Testing**: pytest
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Single project (library module)
**Performance Goals**: 95% of fetches complete within 30 seconds; validation <100ms
**Constraints**: 10MB response size limit, 30-second timeout, HTTP/HTTPS only
**Scale/Scope**: Single-user CLI tool, processing one URL at a time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Readable & Simple Code | ✅ PASS | Single-purpose module with clear input/output |
| II. AHA Programming | ✅ PASS | No abstractions planned; direct implementation |
| III. Minimal Dependencies | ✅ PASS | Only `requests` (well-established) + `idna` (standard for IDN) |
| IV. Accessibility First | ✅ N/A | No UI in this feature |
| V. Monorepo + Open Source | ✅ PASS | Single repo, all open source tools |
| VI. Local First | ✅ PASS | Operates locally, no cloud dependencies |

**Gate Result**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/001-url-ingestion/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── url_ingestion/
│   ├── __init__.py      # Module exports
│   ├── models.py        # FetchResult, FetchError, NormalizedURL
│   ├── validator.py     # URL validation (RFC 3986, protocol check)
│   ├── normalizer.py    # URL normalization (lowercase, IDN, ports)
│   └── fetcher.py       # HTTP fetching with redirects, timeouts, size limits

tests/
├── unit/
│   ├── test_validator.py
│   ├── test_normalizer.py
│   └── test_fetcher.py
└── integration/
    └── test_url_ingestion.py
```

**Structure Decision**: Single project structure selected. The URL ingestion feature is a
standalone library module with no frontend or API layer. Tests are split into unit tests
(for individual components) and integration tests (for end-to-end ingestion flow).

## Complexity Tracking

> No violations - table intentionally empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none)    | -          | -                                   |
