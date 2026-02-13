# Tasks: Test Coverage

**Input**: Design documents from `/specs/007-test-coverage/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/coverage-matrix.md

**Organization**: Tasks are grouped by user story. Since this feature IS about writing tests, each task is a test-writing task.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Test Fixtures & Infrastructure)

**Purpose**: Create shared test fixtures and verify existing tests still pass

- [ ] T001 Run `cargo test` in `src-tauri/` to confirm all 163 existing tests pass (baseline)
- [ ] T002 Create JSON-LD recipe fixture file at `src-tauri/tests/fixtures/jsonld_recipe.html` with "Classic Chocolate Chip Cookies" recipe (all fields populated: title, description, 6 ingredients, 4 HowToStep instructions, PT15M prep, PT12M cook, "24 cookies" servings, 1 image URL, nutrition with calories/fat/carbs/protein) per data-model.md
- [ ] T003 [P] Create Microdata recipe fixture file at `src-tauri/tests/fixtures/microdata_recipe.html` with same recipe content as T002 encoded using `itemscope itemtype="https://schema.org/Recipe"` markup
- [ ] T004 [P] Create no-recipe fixture file at `src-tauri/tests/fixtures/no_recipe.html` with basic HTML page (nav, article, footer) containing no recipe schema markup
- [ ] T005 [P] Create malformed JSON-LD fixture file at `src-tauri/tests/fixtures/malformed_jsonld.html` with broken JSON-LD `<script>` tag (invalid JSON syntax)
- [ ] T006 [P] Create robots.txt fixture files at `src-tauri/tests/fixtures/`: `robots_allow.txt` (Allow: / for all agents), `robots_disallow.txt` (Disallow: / for RecipeScraper), `robots_crawl_delay.txt` (Crawl-delay: 5 for wildcard agent)

**Checkpoint**: All fixtures created. Existing tests still pass.

---

## Phase 2: User Story 1 — Full Pipeline Confidence (Priority: P1)

**Goal**: Integration tests that exercise extract → tag → persist pipeline with fixture HTML, verifying cross-module data integrity.

**Independent Test**: `cargo test --test pipeline` passes with known fixture inputs producing expected stored outputs.

- [ ] T007 [US1] Create integration test file at `src-tauri/tests/pipeline_test.rs` with a test helper function that creates an in-memory `Database` (using `Database::new_in_memory()` pattern from repository.rs tests), runs migrations, and returns the database. Include `include_str!()` macros to load fixtures from `tests/fixtures/`.
- [ ] T008 [US1] Add integration test `test_jsonld_extract_tag_persist_roundtrip` in `src-tauri/tests/pipeline_test.rs`: load JSON-LD fixture → call `extract_recipe()` → assert title is "Classic Chocolate Chip Cookies" with 6 ingredients → call `tag_recipe_from_extracted()` → assert course tags include "dessert" → call `save_recipe()` on in-memory DB → call `get_recipe()` → assert all fields match extraction output and tags are preserved.
- [ ] T009 [US1] Add integration test `test_microdata_extract_tag_persist_roundtrip` in `src-tauri/tests/pipeline_test.rs`: load Microdata fixture → call `extract_recipe()` → assert same title/ingredients as JSON-LD → call `tag_recipe_from_extracted()` → assert same tags → call `save_recipe()` → call `get_recipe()` → assert all fields match.
- [ ] T010 [US1] Add integration test `test_no_recipe_html_returns_extraction_error` in `src-tauri/tests/pipeline_test.rs`: load no-recipe fixture → call `extract_recipe()` → assert returns `ExtractionError::NoRecipeFound`.
- [ ] T011 [US1] Add integration test `test_malformed_jsonld_returns_error` in `src-tauri/tests/pipeline_test.rs`: load malformed JSON-LD fixture → call `extract_recipe()` → assert returns an error variant (not a panic).
- [ ] T012 [US1] Add unit test `test_ingest_url_rejects_robots_disallowed` in `src-tauri/src/url_ingestion/commands.rs`: create in-memory DB → insert a robots_cache entry for "example.com" with status "disallowed" and `raw_content` containing `Disallow: /` → call the internal compliance check function with `http://example.com/recipe` → assert it returns `RobotsDecision { allowed: false }`. Verify the URL is blocked before any fetch attempt.
- [ ] T013 [US1] Run `cargo test` in `src-tauri/` to verify all existing tests plus new integration tests pass without regressions.

**Checkpoint**: Pipeline integration tests validate extract → tag → persist roundtrip and robots gate. `cargo test` passes.

---

## Phase 3: User Story 2 — Untested Critical Paths (Priority: P1)

**Goal**: Add tests for backup/restore and change log modules that currently have zero coverage.

**Independent Test**: `cargo test storage::backup` and `cargo test storage::change_log` pass with at least 3 test cases each.

### Backup Tests

- [ ] T014 [P] [US2] Add `#[cfg(test)] mod tests` to `src-tauri/src/storage/backup.rs` with test helper that creates an in-memory DB, runs migrations, and populates it with 3 test recipes (using the existing `save_recipe` from repository.rs). Add test `test_backup_roundtrip`: create populated DB → call `backup_collection_to()` with a temp file path → open backup file as new `Connection` → verify `SELECT COUNT(*) FROM recipes WHERE deleted = 0` equals 3 → clean up temp file.
- [ ] T015 [P] [US2] Add test `test_restore_from_backup` in `src-tauri/src/storage/backup.rs`: create populated source DB → backup to temp file → create fresh empty DB (in-memory, with migrations) → call `restore_collection_from()` with backup path → call `get_recipe()` for each recipe ID → assert all fields match originals → clean up temp file.
- [ ] T016 [P] [US2] Add test `test_restore_corrupted_backup_returns_error` in `src-tauri/src/storage/backup.rs`: create an in-memory DB → write invalid bytes ("not a database") to a temp file → call `restore_collection_from()` → assert it returns `Err(StorageError::...)` → verify original DB is unchanged (can still query existing data) → clean up temp file.
- [ ] T017 [US2] Add test `test_restore_empty_file_returns_error` in `src-tauri/src/storage/backup.rs`: create an in-memory DB with 1 recipe → create empty temp file (0 bytes) → call `restore_collection_from()` → assert it returns error → verify original recipe still accessible → clean up temp file.

### Change Log Tests

- [ ] T018 [P] [US2] Add `#[cfg(test)] mod tests` to `src-tauri/src/storage/change_log.rs` with test `test_append_change_creates_entry`: create in-memory DB with migrations → call `append_change(conn, "recipe-123", "created", None, "device-1")` → query `change_log` table directly → assert 1 row with matching recipe_id, field_name, device_id, and non-empty modified_at timestamp.
- [ ] T019 [P] [US2] Add test `test_query_pending_returns_unsynced_entries` in `src-tauri/src/storage/change_log.rs`: create DB → append 3 changes for different recipe IDs → call `query_pending(conn)` → assert returns 3 `ChangeEntry` items with correct recipe_id, field_name, and sequential IDs.
- [ ] T020 [P] [US2] Add test `test_mark_synced_clears_entries` in `src-tauri/src/storage/change_log.rs`: create DB → append 3 changes → get max ID from `query_pending()` → call `mark_synced(conn, max_id)` → call `query_pending()` again → assert returns empty vec.
- [ ] T021 [US2] Add test `test_now_utc_returns_valid_iso8601` in `src-tauri/src/storage/change_log.rs`: call `now_utc()` → assert string matches ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ pattern) → assert year >= 2026 (sanity check).
- [ ] T022 [US2] Run `cargo test storage::backup` and `cargo test storage::change_log` to verify all new tests pass, then run full `cargo test` for regression check.

**Checkpoint**: Backup (4 tests) and change log (4 tests) modules now have comprehensive coverage. `cargo test` passes.

---

## Phase 4: User Story 3 — Spec-Aligned Boundary Tests (Priority: P2)

**Goal**: Fill every remaining gap in the coverage matrix — ensure each spec acceptance scenario has at least one test.

**Independent Test**: Coverage matrix shows all scenarios as "Existing" or covered by new tests; `cargo test` passes.

### Recipe Extraction Edge Cases (Spec 003)

- [ ] T023 [P] [US3] Add test `test_jsonld_missing_prep_time_returns_not_found` in `src-tauri/src/recipe_extraction/json_ld.rs`: create JSON-LD HTML with Recipe schema but no `prepTime` field → call `extract_from_jsonld()` → assert `prep_time_minutes` is `ExtractedField::NotFound` with non-empty justification string.
- [ ] T024 [P] [US3] Add test `test_jsonld_missing_nutrition_returns_not_found` in `src-tauri/src/recipe_extraction/json_ld.rs`: create JSON-LD HTML with Recipe schema but no `nutrition` field → call `extract_from_jsonld()` → assert `nutrition` is `ExtractedField::NotFound` with justification.
- [ ] T025 [P] [US3] Add test `test_jsonld_missing_images_returns_not_found` in `src-tauri/src/recipe_extraction/json_ld.rs`: create JSON-LD HTML with Recipe schema but no `image` field → call `extract_from_jsonld()` → assert `images` is `ExtractedField::NotFound` with justification.
- [ ] T026 [P] [US3] Add test `test_jsonld_multiple_images_all_captured` in `src-tauri/src/recipe_extraction/json_ld.rs`: create JSON-LD HTML with `"image": ["url1.jpg", "url2.jpg", "url3.jpg"]` → call `extract_from_jsonld()` → assert `images` is `ExtractedField::Found` containing all 3 URLs.
- [ ] T027 [P] [US3] Add test `test_jsonld_nutrition_fields_extracted` in `src-tauri/src/recipe_extraction/json_ld.rs`: create JSON-LD HTML with full `nutrition` object (calories, fat, carbs, protein) → call `extract_from_jsonld()` → assert `nutrition` is `ExtractedField::Found` with correct values parsed.
- [ ] T028 [P] [US3] Add test `test_jsonld_malformed_falls_back_gracefully` in `src-tauri/src/recipe_extraction/json_ld.rs`: create HTML with `<script type="application/ld+json">{ "broken": }</script>` → call `extract_from_jsonld()` → assert returns error (not panic).

### Microdata Edge Cases (Spec 003)

- [ ] T029 [P] [US3] Add test `test_microdata_missing_optional_fields_returns_not_found` in `src-tauri/src/recipe_extraction/microdata.rs`: create Microdata HTML with Recipe itemscope containing only title and ingredients (no prepTime, no nutrition, no image) → call `extract_from_microdata()` → assert `prep_time_minutes`, `nutrition`, and `images` are `ExtractedField::NotFound` with justification strings.

### URL Ingestion Edge Cases (Specs 001/002)

- [ ] T030 [P] [US3] Add test `test_oversized_response_rejected` in `src-tauri/src/url_ingestion/fetcher.rs`: test the size-checking logic by verifying that a response body exceeding 10MB (10_485_760 bytes) produces a `FetchError::Size` error. Use inline bytes or the existing size-check function directly.
- [ ] T031 [P] [US3] Add test `test_non_html_content_type_rejected` in `src-tauri/src/url_ingestion/fetcher.rs`: test the content-type validation logic by verifying that a response with `Content-Type: application/json` (or similar non-HTML type) produces a `FetchError::ContentType` error.

### Robots Compliance Edge Cases (Spec 006)

- [ ] T032 [P] [US3] Add test `test_empty_robots_txt_allows_all` in `src-tauri/src/robots_compliance/checker.rs`: test the compliance decision logic with an empty string as `raw_content` and status "found" → assert `allowed` is `true` (empty robots.txt = no restrictions).
- [ ] T033 [P] [US3] Add test `test_disallowed_path_returns_blocked` in `src-tauri/src/robots_compliance/checker.rs`: test with `raw_content` containing `User-agent: RecipeScraper\nDisallow: /recipes` and a URL path `/recipes/page` → assert `allowed` is `false` and `reason` contains "disallowed".
- [ ] T034 [P] [US3] Add test `test_wildcard_disallow_blocks_all` in `src-tauri/src/robots_compliance/checker.rs`: test with `raw_content` containing `User-agent: *\nDisallow: /` → assert `allowed` is `false`.
- [ ] T035 [US3] Add test `test_malformed_robots_parses_valid_lines` in `src-tauri/src/robots_compliance/crawl_delay.rs`: create robots.txt with mix of valid and invalid lines (e.g., `Crawl-delay: abc\nCrawl-delay: 3`) → call `parse_crawl_delay()` → assert returns `Some(3.0)` (ignores invalid, uses valid).
- [ ] T036 [US3] Run `cargo test recipe_extraction` and `cargo test robots_compliance` and `cargo test url_ingestion` to verify new edge case tests pass, then full `cargo test` for regression check.

**Checkpoint**: All spec acceptance scenarios from coverage matrix now have corresponding tests. `cargo test` passes.

---

## Phase 5: User Story 4 — Command Layer Verification (Priority: P3)

**Goal**: Verify Tauri command wrappers correctly forward arguments, serialize responses, and propagate errors.

**Independent Test**: `cargo test commands` passes across all modules.

### Storage Commands

- [ ] T037 [P] [US4] Add `#[cfg(test)] mod tests` to `src-tauri/src/storage/commands.rs` with test `test_save_recipe_command_returns_save_result`: create in-memory DB → construct a minimal `ExtractedRecipe` and `TagSet` → call the save logic (the function that `save_recipe` command delegates to) → assert returns `SaveResult` with non-empty `id` and `created: true`.
- [ ] T038 [P] [US4] Add test `test_get_recipe_not_found_returns_error` in `src-tauri/src/storage/commands.rs`: create in-memory DB → call get logic with non-existent ID → assert returns `StorageError::NotFound`.
- [ ] T039 [P] [US4] Add test `test_delete_recipe_returns_result` in `src-tauri/src/storage/commands.rs`: create DB → save a recipe → call delete logic → assert returns `DeleteResult { deleted: true }` → call get → assert `NotFound`.

### URL Ingestion Commands

- [ ] T040 [P] [US4] Add `#[cfg(test)] mod tests` to `src-tauri/src/url_ingestion/commands.rs` (if not already present) with test `test_validate_url_valid_returns_normalized`: call `validate_url` internal logic with `"https://EXAMPLE.COM/path/"` → assert returns `Ok(NormalizedUrl)` with lowercase host and trimmed trailing slash.
- [ ] T041 [P] [US4] Add test `test_validate_url_invalid_returns_error` in `src-tauri/src/url_ingestion/commands.rs`: call validate logic with `"not-a-url"` → assert returns `Err(FetchError::Validation { .. })`.

### Robots Compliance Commands

- [ ] T042 [P] [US4] Add `#[cfg(test)] mod tests` to `src-tauri/src/robots_compliance/commands.rs` with test `test_check_robots_invalid_url_returns_error`: call the compliance check logic with `"not-a-url"` → assert returns `Err(RobotsError::InvalidUrl { .. })`.

### Recipe Tagging Commands

- [ ] T043 [P] [US4] Add `#[cfg(test)] mod tests` to `src-tauri/src/recipe_tagging/commands.rs` (if not already present) with test `test_tag_recipe_command_returns_tagset`: construct a minimal `ExtractedRecipe` with title "Pad Thai" and ingredients including "fish sauce", "rice noodles" → call `tag_recipe_from_extracted()` → assert returns `TagSet` with non-empty `cuisine` vec containing a Thai-related tag.
- [ ] T044 [P] [US4] Add test `test_extract_and_tag_no_recipe_returns_error` in `src-tauri/src/recipe_tagging/commands.rs`: call `extract_and_tag` logic with HTML containing no recipe markup → assert returns `Err(TaggingError::ExtractionFailed { .. })`.

- [ ] T045 [US4] Run `cargo test commands` across all modules and full `cargo test` for regression check.

**Checkpoint**: Command layer tests verify argument forwarding and error propagation. `cargo test` passes.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, formatting, and regression checks

- [ ] T046 Run `cargo clippy` in `src-tauri/` and fix any warnings in new test code
- [ ] T047 Run `cargo fmt` in `src-tauri/` to ensure consistent formatting
- [ ] T048 Run full `cargo test` in `src-tauri/` and verify total test count increased by ~40+ new tests with 0 failures, completing in under 60 seconds
- [ ] T049 Update coverage matrix at `specs/007-test-coverage/contracts/coverage-matrix.md` — change all "New" entries to "Existing" with actual test function names and locations

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **US1 (Phase 2)**: Depends on Phase 1 fixtures (T002-T006)
- **US2 (Phase 3)**: No dependency on Phase 1 fixtures — uses in-memory DBs and inline data
- **US3 (Phase 4)**: No dependency on fixtures — uses inline HTML snippets
- **US4 (Phase 5)**: No dependency on fixtures — uses inline data
- **Polish (Phase 6)**: Depends on all user stories (Phases 2-5) being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on fixtures (Phase 1). No dependency on other stories.
- **User Story 2 (P1)**: Independent — can start in parallel with US1 after T001 baseline passes.
- **User Story 3 (P2)**: Independent — can start in parallel with US1/US2 after T001 baseline.
- **User Story 4 (P3)**: Independent — can start in parallel with all other stories after T001 baseline.

### Within Each User Story

- Tasks marked [P] within a story can run in parallel
- Final regression check task in each story must run last
- Commit after each story completion

### Parallel Opportunities

- T002-T006 (fixture creation) can all run in parallel
- US2, US3, US4 can all start in parallel with US1 (only US1 needs fixtures)
- Within US2: backup tests (T014-T017) and change log tests (T018-T021) can run in parallel
- Within US3: all extraction tests (T023-T029) and all robots tests (T032-T035) and ingestion tests (T030-T031) can run in parallel
- Within US4: all command tests (T037-T044) can run in parallel

---

## Parallel Example: User Story 2

```bash
# Launch backup tests and change log tests in parallel:
Task: "T014 [P] [US2] Add backup roundtrip test in src-tauri/src/storage/backup.rs"
Task: "T018 [P] [US2] Add change log append test in src-tauri/src/storage/change_log.rs"

# After both complete, run regression:
Task: "T022 [US2] Run cargo test for regression check"
```

## Parallel Example: User Story 4

```bash
# Launch all command tests in parallel:
Task: "T037 [P] [US4] Storage save command test in storage/commands.rs"
Task: "T040 [P] [US4] URL validation command test in url_ingestion/commands.rs"
Task: "T042 [P] [US4] Robots compliance command test in robots_compliance/commands.rs"
Task: "T043 [P] [US4] Recipe tagging command test in recipe_tagging/commands.rs"

# After all complete, run regression:
Task: "T045 [US4] Run cargo test for regression check"
```

## Parallel Example: User Story 3

```bash
# Launch all extraction edge cases and robots edge cases in parallel:
Task: "T023 [P] [US3] Missing prep time test in json_ld.rs"
Task: "T024 [P] [US3] Missing nutrition test in json_ld.rs"
Task: "T030 [P] [US3] Oversized response test in fetcher.rs"
Task: "T032 [P] [US3] Empty robots.txt test in checker.rs"

# After all complete, run regression:
Task: "T036 [US3] Run cargo test for regression check"
```

---

## Implementation Strategy

### MVP First (User Story 1 + User Story 2)

1. Complete Phase 1: Setup (fixtures + baseline)
2. Complete Phase 2: US1 — Pipeline integration tests
3. Complete Phase 3: US2 — Backup + change log tests
4. **STOP and VALIDATE**: `cargo test` passes, critical gaps filled
5. This alone satisfies SC-002 and SC-003

### Incremental Delivery

1. Setup + US1 + US2 → Critical coverage gaps filled (MVP)
2. Add US3 → All spec scenarios covered (SC-001)
3. Add US4 → Command layer verified (FR-006)
4. Polish → Formatting, clippy, coverage matrix updated

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each phase completion
- All tests use in-memory databases or fixture files — no network access
- Total new tests: ~45 (7 integration + 8 backup/changelog + 13 boundary + 8 command + regression checks)
- FR-005 "concurrent operations": Deferred — existing `atomic_transaction_safety` test in repository.rs already validates SQLite transaction isolation; additional concurrency tests would require async test harness complexity for low incremental value
