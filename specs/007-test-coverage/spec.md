# Feature Specification: Test Coverage

**Feature Branch**: `007-test-coverage`
**Created**: 2026-02-13
**Status**: Draft
**Input**: User description: "Provide automated tests aligned with specs. Test ingestion, extraction, tag inference, database persistence, and edge cases. All specs have corresponding test suites. Tests reflect expected input/output boundaries."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Full Pipeline Confidence (Priority: P1)

As a developer, I want integration tests that exercise the full recipe pipeline — from URL ingestion through extraction, tagging, and persistence — so that I can be confident changes in one module don't silently break downstream behavior.

**Why this priority**: The existing test suite covers individual modules in isolation but has no tests validating the end-to-end flow. A regression at any integration boundary (e.g., extraction output not matching what storage expects) would go undetected.

**Independent Test**: Can be tested by running the full test suite and verifying that pipeline tests pass with known recipe inputs producing expected stored outputs.

**Acceptance Scenarios**:

1. **Given** a valid URL pointing to a page with JSON-LD recipe markup, **When** the pipeline processes it end-to-end, **Then** the extracted recipe is tagged and persisted with all fields intact.
2. **Given** a URL that is blocked by robots.txt, **When** the pipeline processes it, **Then** the request is rejected before any fetch occurs, and no data is stored.
3. **Given** a URL pointing to a page with Microdata recipe markup, **When** the pipeline processes it, **Then** the recipe is correctly extracted, tagged, and stored identically to JSON-LD for equivalent content.

---

### User Story 2 - Untested Critical Paths (Priority: P1)

As a developer, I want tests for currently untested critical modules — backup/restore and change log — so that data safety and sync reliability are verified before users rely on them.

**Why this priority**: Backup/restore is a data-safety feature with zero test coverage. Change log drives sync behavior. Both handle user data and failures here could cause data loss.

**Independent Test**: Can be tested by creating a database with known state, running backup/restore, and verifying data roundtrip integrity. Change log tests verify append and read operations produce correct entries.

**Acceptance Scenarios**:

1. **Given** a database with stored recipes, **When** a backup is created and restored to a fresh database, **Then** all recipes and metadata are identical to the original.
2. **Given** a backup file that is corrupted or empty, **When** a restore is attempted, **Then** the operation fails gracefully with a clear error and the existing database is unchanged.
3. **Given** a series of recipe create/update/delete operations, **When** the change log is queried, **Then** each operation is recorded with correct type, timestamp, and affected recipe identifier.

---

### User Story 3 - Spec-Aligned Boundary Tests (Priority: P2)

As a developer, I want each feature spec's acceptance criteria to have a corresponding test, so that the spec serves as a living contract verified by the test suite.

**Why this priority**: Existing tests cover many happy paths but don't systematically trace back to spec acceptance criteria. Gaps exist for edge cases defined in specs (e.g., IDN/Punycode URLs, oversized HTML responses, empty ingredient lists).

**Independent Test**: Can be tested by mapping each spec's acceptance scenarios to test cases and verifying all are present and passing.

**Acceptance Scenarios**:

1. **Given** the acceptance scenarios in specs 001-006, **When** the test suite is reviewed, **Then** every scenario has at least one corresponding automated test.
2. **Given** edge cases documented in each spec, **When** those edge case inputs are provided, **Then** the system handles them as described in the spec.

---

### User Story 4 - Command Layer Verification (Priority: P3)

As a developer, I want tests for Tauri command wrappers that verify argument handling, error propagation, and response shaping, so that the frontend integration contract is validated.

**Why this priority**: Command wrappers are thin but they are the API surface consumed by the frontend. Incorrect serialization, missing error variants, or mismatched field names would only surface at runtime.

**Independent Test**: Can be tested by invoking command functions directly (without Tauri runtime) and verifying inputs are forwarded correctly and outputs match expected shapes.

**Acceptance Scenarios**:

1. **Given** a valid input to any Tauri command, **When** the command is invoked, **Then** the response matches the documented contract shape.
2. **Given** an invalid input to a Tauri command, **When** the command is invoked, **Then** the error response contains the expected error variant and message.

---

### Edge Cases

- What happens when tests depend on network access (e.g., fetcher, robots checker)? All network-dependent tests must be clearly separated and skippable for offline development.
- What happens when the test database file is locked or inaccessible? Tests using SQLite must create isolated temporary databases.
- What happens when test fixtures contain malformed data (invalid JSON-LD, broken HTML)? These should be covered as negative test cases.
- What happens when multiple tests write to the same database concurrently? Each test must use its own isolated database instance.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Test suite MUST include integration tests that exercise the pipeline from URL ingestion through recipe storage.
- **FR-002**: Test suite MUST cover backup creation, backup restore (valid and corrupted), and verify data roundtrip integrity.
- **FR-003**: Test suite MUST cover change log operations — append on create/update/delete and retrieval with correct metadata.
- **FR-004**: Test suite MUST include tests for every acceptance scenario defined in specs 001 through 006.
- **FR-005**: Test suite MUST cover edge cases: IDN/Punycode URLs, oversized responses, empty/missing recipe fields, malformed markup, concurrent operations.
- **FR-006**: Test suite MUST verify Tauri command wrappers for correct argument forwarding, response serialization, and error propagation.
- **FR-007**: Test suite MUST use isolated temporary databases for any test that writes data, preventing cross-test contamination.
- **FR-008**: Test suite MUST separate network-dependent tests so they can be skipped in offline environments.
- **FR-009**: All existing tests MUST continue to pass without modification (no regressions from adding new tests).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every acceptance scenario across specs 001-006 has at least one corresponding automated test.
- **SC-002**: All previously untested critical paths (backup, restore, change log) have at least 3 test cases each covering happy path, error case, and edge case.
- **SC-003**: At least one end-to-end integration test validates the full pipeline (ingest → extract → tag → persist) with known fixture data.
- **SC-004**: The full test suite passes in under 60 seconds on a standard development machine.
- **SC-005**: No test requires network access unless explicitly marked and skippable.

## Assumptions

- Tests will use fixture data (static HTML/JSON files) rather than live network requests for reproducibility.
- The existing inline `#[cfg(test)]` pattern will be extended rather than introducing a separate test framework.
- Tauri command tests will invoke the command functions directly without requiring the Tauri runtime, since the commands are thin wrappers.
- "All specs" refers to specs 001 through 006 (the implemented features at time of writing).
