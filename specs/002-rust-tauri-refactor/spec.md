# Feature Specification: Rust/Tauri Backend Refactor

**Feature Branch**: `002-rust-tauri-refactor`
**Created**: 2026-02-04
**Status**: Draft
**Input**: User description: "refactor python backend to rust. create rust skills files. aim for compatibility with Tauri. Maintain local first principles. Remove all old python code"

## Clarifications

### Session 2026-02-04

- Q: Should Tauri commands be async or sync? → A: Async commands - Frontend receives Promise, UI stays responsive during fetch

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fetch Valid Recipe URL (Priority: P1)

As a user, I want to provide a recipe URL and receive the HTML content so that I can later extract recipe data from it. The system validates and normalizes the URL before fetching, ensuring consistent behavior across different URL formats.

**Why this priority**: This is the core MVP functionality - without URL fetching, no recipe scraping can occur. This replicates the existing Python functionality in Rust.

**Independent Test**: Can be fully tested by providing a known valid URL (e.g., https://httpbin.org/html) and verifying that HTML content is returned successfully.

**Acceptance Scenarios**:

1. **Given** a valid HTTPS URL, **When** I request to ingest the URL, **Then** the system returns the HTML content with status code and content type
2. **Given** a URL with uppercase characters or non-standard formatting, **When** I request to ingest the URL, **Then** the system normalizes it (lowercase host, removes default ports) before fetching
3. **Given** an internationalized domain name (IDN), **When** I request to ingest the URL, **Then** the system converts it to Punycode for fetching

---

### User Story 2 - Handle Invalid URLs (Priority: P2)

As a user, when I provide an invalid URL, I want clear error messages explaining what's wrong so that I can correct the input.

**Why this priority**: Error handling is essential for a good user experience, but the happy path (US1) must work first.

**Independent Test**: Can be tested by providing various invalid inputs (empty string, missing scheme, wrong protocol) and verifying appropriate error responses.

**Acceptance Scenarios**:

1. **Given** an empty or whitespace-only input, **When** I request to ingest, **Then** the system returns a validation error with message "No URL provided"
2. **Given** a URL without a scheme (e.g., "example.com"), **When** I request to ingest, **Then** the system returns a validation error indicating the missing scheme
3. **Given** a non-HTTP/HTTPS URL (e.g., "ftp://..."), **When** I request to ingest, **Then** the system returns a validation error rejecting the protocol

---

### User Story 3 - Handle Network Failures (Priority: P3)

As a user, when a URL cannot be reached due to network issues or server errors, I want structured error information so that I can understand and potentially retry the request.

**Why this priority**: Network error handling completes the robustness of the module but depends on the fetch functionality from US1.

**Independent Test**: Can be tested by providing unreachable URLs (non-existent domains, URLs returning 404/500) and verifying structured error responses.

**Acceptance Scenarios**:

1. **Given** a URL with an unresolvable domain, **When** I request to ingest, **Then** the system returns a network error indicating DNS failure
2. **Given** a URL that times out, **When** I request to ingest, **Then** the system returns a network error after 30 seconds
3. **Given** a URL returning HTTP 4xx/5xx, **When** I request to ingest, **Then** the system returns an HTTP error with the status code
4. **Given** a URL returning non-HTML content, **When** I request to ingest, **Then** the system returns a content-type error
5. **Given** a response exceeding 10MB, **When** I request to ingest, **Then** the system returns a size error

---

### User Story 4 - Tauri Integration (Priority: P4)

As a developer building a Tauri desktop application, I want the URL ingestion module exposed as Tauri commands so that the frontend can invoke backend functionality.

**Why this priority**: Tauri integration enables the desktop app use case but requires the core functionality (US1-3) to exist first.

**Independent Test**: Can be tested by invoking Tauri commands from a minimal frontend and verifying responses match the expected format.

**Acceptance Scenarios**:

1. **Given** a Tauri application context, **When** I call the `ingest_url` command with a valid URL, **Then** I receive a JSON response with the fetch result
2. **Given** a Tauri application context, **When** I call the `validate_url` command, **Then** I receive a JSON response with validation result without fetching

---

### User Story 5 - Remove Python Code (Priority: P5)

As a project maintainer, I want all Python code removed after the Rust implementation is complete to maintain a clean, single-language codebase.

**Why this priority**: Cleanup happens after the Rust implementation fully replaces Python functionality.

**Independent Test**: Can be verified by checking that no `.py` files exist in `src/` and `tests/`, and that Python-related configuration (pyproject.toml) is removed.

**Acceptance Scenarios**:

1. **Given** the Rust implementation is complete and tested, **When** the refactor is finalized, **Then** all Python files in `src/` and `tests/` are deleted
2. **Given** Python removal is complete, **When** I check the repository, **Then** pyproject.toml and Python-specific configurations are removed

---

### Edge Cases

- What happens when the URL contains unusual but valid characters (e.g., emoji in path)?
- How does the system handle redirects that change protocol (HTTP to HTTPS)?
- What happens when Content-Length header is missing for large responses?
- How does the system handle connection resets mid-transfer?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST validate URLs using the same rules as the Python implementation (HTTP/HTTPS only, valid syntax)
- **FR-002**: System MUST normalize URLs identically to Python (lowercase scheme/host, remove default ports 80/443, IDN to Punycode)
- **FR-003**: System MUST fetch HTML content with 30-second timeout and 10MB size limit
- **FR-004**: System MUST follow up to 5 redirects when fetching URLs
- **FR-005**: System MUST send "RecipeScraper/1.0" as User-Agent header
- **FR-006**: System MUST return structured success results containing: normalized URL, HTML content, status code, content type, final URL if redirected
- **FR-007**: System MUST return structured error results with error type (validation, network, HTTP, content-type, size), message, original URL, and optional details
- **FR-008**: System MUST expose `ingest_url` and `validate_url` functions as async Tauri commands returning Promises to the frontend
- **FR-009**: System MUST store data locally without requiring cloud services (local-first principle)
- **FR-010**: System MUST provide Claude Code skill files for Rust development patterns

### Key Entities

- **NormalizedUrl**: A validated URL with scheme, host, optional port, path, optional query, optional fragment
- **FetchSuccess**: Successful result containing NormalizedURL, HTML string, status code, content type, optional final URL
- **FetchError**: Error result containing error type enum, human-readable message, original URL, optional details map
- **ErrorType**: Enumeration of error categories (Validation, Network, Http, ContentType, Size)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All existing Python URL ingestion acceptance scenarios pass with the Rust implementation (verified manually per T038)
- **SC-002**: URL validation completes in under 10 milliseconds for typical URLs
- **SC-003**: 95% of fetches complete within 30 seconds (matching timeout behavior)
- **SC-004**: Error messages are distinguishable by type without inspecting error details
- **SC-005**: Tauri commands can be invoked from JavaScript/TypeScript frontend code
- **SC-006**: Application works fully offline for local operations (local-first compliance)
- **SC-007**: Zero Python files remain in src/ and tests/ directories after completion
- **SC-008**: Claude Code Rust skill files are created and functional

## Assumptions

- Rust stable toolchain (1.70+) is available in the development environment
- Tauri v2 will be used for the desktop application framework
- The existing Python test scenarios serve as the acceptance criteria for functional parity
- No database or persistent storage is needed for this module (stateless URL fetching)
- IDN/Punycode support will use the `idna` crate (Rust equivalent of Python's idna)
- HTTP client will use `reqwest` crate (Rust equivalent of Python's requests)

## Out of Scope

- Recipe parsing/extraction (separate future feature)
- Caching of fetched content
- Rate limiting or politeness delays
- JavaScript rendering (Playwright equivalent)
- Batch URL processing
- User interface implementation (frontend is separate concern)
- Edge case handling beyond standard library/crate behavior (emoji paths, protocol-changing redirects, missing Content-Length, mid-transfer resets)
