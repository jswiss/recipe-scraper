# Research: Test Coverage

## R-001: Test Organization Pattern

**Decision**: Inline `#[cfg(test)]` modules for unit tests; `tests/` directory for integration tests.

**Rationale**: The codebase already uses inline test modules consistently across 20 files. Rust's convention is `tests/` for integration tests that exercise the public API. Adding integration tests there maintains idiomatic project structure without disrupting existing patterns.

**Alternatives considered**:
- Separate `tests/unit/` directory for all tests: Rejected — would require restructuring 163 existing tests and break convention.
- Test-only crate: Rejected — unnecessary complexity for a single application.

## R-002: Test Fixture Strategy

**Decision**: Static HTML/JSON/txt fixture files in `src-tauri/tests/fixtures/`. Loaded at compile time via `include_str!()` for integration tests; inline string literals for unit tests that need only small snippets.

**Rationale**: `include_str!()` embeds fixture content at compile time, eliminating runtime file I/O and path resolution issues. Fixture files are human-readable and editable. Unit tests that need only a few lines of HTML can use inline strings (consistent with existing test patterns in json_ld.rs and microdata.rs).

**Alternatives considered**:
- Runtime file reads with `std::fs::read_to_string`: Rejected — introduces path dependency on working directory, fragile in CI.
- All inline strings: Rejected — full HTML fixtures would be unreadable as inline strings.
- Snapshot testing (insta crate): Rejected — adds a dependency (violates Principle III) for minimal benefit.

## R-003: Database Isolation for Tests

**Decision**: Use `rusqlite::Connection::open_in_memory()` for all database-dependent tests. Each test creates its own in-memory database, runs migrations, and operates independently.

**Rationale**: In-memory databases are fast, automatically cleaned up, and guarantee no cross-test contamination. The existing `repository.rs` tests already use this pattern successfully with 23 tests. The `Database` struct wraps a `Mutex<Connection>`, so tests can create a `Database` from an in-memory connection.

**Alternatives considered**:
- Temp files with `tempfile` crate: Rejected — slower, requires cleanup, adds dependency.
- Shared test database with transactions: Rejected — risk of contamination, harder to parallelize.

## R-004: Testing Tauri Commands Without Tauri Runtime

**Decision**: Test command functions directly by calling them with manually constructed arguments. Use `Database` and `HttpClient` created in test setup rather than Tauri's `State<>` wrapper.

**Rationale**: Tauri commands are thin wrappers that accept `State<T>` parameters. The `State<T>` type dereferences to `&T`, so tests can pass `&Database` directly to the underlying logic functions. The commands themselves just forward to module-internal functions, so testing the internal functions provides equivalent coverage. For commands that do add logic (like `ingest_url` which gates on robots compliance), the internal function can be tested directly.

**Alternatives considered**:
- Full Tauri test harness (`tauri::test::mock_builder`): Rejected — adds significant complexity, slower, and the commands are thin enough that direct function testing suffices.
- Mocking `State<T>`: Rejected — unnecessary when we can test the underlying functions directly.

## R-005: Network Test Separation

**Decision**: No network tests in this feature. All tests use fixture data. The fetcher module's existing tests only cover error message formatting (no actual HTTP calls), and that pattern continues.

**Rationale**: FR-008 requires network-dependent tests to be skippable. Rather than adding network tests that need special handling, all new tests use fixtures. Real HTTP fetching is already validated manually during development and would require a test HTTP server (added complexity for low value).

**Alternatives considered**:
- Mock HTTP server (wiremock/mockito crate): Rejected — adds dependency for minimal value; existing fetcher tests don't use network.
- `#[ignore]` attribute for network tests: Not needed since we're not adding network tests.

## R-006: Integration Test Scope

**Decision**: Integration tests exercise the extract → tag → persist pipeline using fixture HTML. The ingest step (HTTP fetch) is bypassed since it requires network; tests start from HTML content as if already fetched.

**Rationale**: The valuable integration boundary is between extraction, tagging, and storage. URL validation and normalization are well-covered by unit tests. The robots compliance gate in `ingest_url` can be tested at the unit level by calling `check_compliance` with a pre-populated cache.

**Alternatives considered**:
- Full ingest → extract → tag → persist with mock HTTP: Rejected — high complexity, and each segment is well-tested in isolation.
- Only unit tests, no integration: Rejected — misses cross-module boundary bugs (e.g., extraction output shape not matching storage expectations).

## R-007: Backup/Restore Test Approach

**Decision**: Test `backup_collection_to` and `restore_collection_from` using in-memory source databases and temp file paths for backup files. Validate roundtrip integrity by comparing recipe counts and field values.

**Rationale**: `rusqlite::backup::Backup` works between any two `Connection` instances, including in-memory ones as source. The backup destination must be a file path (SQLite backup API requirement). Tests will use `tempfile::NamedTempFile` — the only place we need temp files since backup literally writes to disk.

**Alternatives considered**:
- In-memory to in-memory backup: Not supported by SQLite backup API (destination must be file).
- Skip file validation: Rejected — the corrupted backup test case requires a real file.

**Note**: `tempfile` crate is NOT needed as a dependency — tests can use `std::env::temp_dir()` with a unique name to create temp files, then clean up in test teardown.
