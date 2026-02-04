# Tasks: Rust/Tauri Backend Refactor

**Input**: Design documents from `/specs/002-rust-tauri-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in the feature specification. Test tasks omitted.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Tauri project**: `src-tauri/` at repository root
- Rust module: `src-tauri/src/url_ingestion/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Tauri project initialization and Rust dependencies

- [ ] T001 Initialize Tauri v2 project with `cargo tauri init` creating `src-tauri/` directory structure
- [ ] T002 Configure `src-tauri/Cargo.toml` with dependencies: tauri 2.x, reqwest 0.12+ (features: json, rustls-tls), url 2.x, idna 1.x, serde 1.x (features: derive), serde_json 1.x, thiserror 2.x, tokio 1.x
- [ ] T003 [P] Configure `src-tauri/tauri.conf.json` with app identifier, window settings, and security permissions
- [ ] T004 [P] Create `src-tauri/src/url_ingestion/mod.rs` with module declarations (models, validator, normalizer, fetcher, commands)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data models and error types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Create ErrorType enum in `src-tauri/src/url_ingestion/models.rs` with variants: Validation, Network, Http, ContentType, Size (with serde rename_all = "snake_case")
- [ ] T006 Create NormalizedUrl struct in `src-tauri/src/url_ingestion/models.rs` with fields: scheme, host, port (Option<u16>), path, query (Option), fragment (Option), and impl Display for URL reconstruction
- [ ] T007 Create FetchSuccess struct in `src-tauri/src/url_ingestion/models.rs` with fields: url (NormalizedUrl), html (String), status_code (u16), content_type (String), final_url (Option<String>)
- [ ] T008 Create FetchError enum in `src-tauri/src/url_ingestion/models.rs` with variants matching ErrorType, each containing message, url, and type-specific fields (using thiserror derive)
- [ ] T009 Create FetchResult type alias (Result<FetchSuccess, FetchError>) in `src-tauri/src/url_ingestion/models.rs`
- [ ] T010 [P] Export all public types from `src-tauri/src/url_ingestion/mod.rs`

**Checkpoint**: Foundation ready - all data models defined, user story implementation can begin

---

## Phase 3: User Story 1 - Fetch Valid Recipe URL (Priority: P1) 🎯 MVP

**Goal**: Accept valid URLs, normalize them, fetch HTML content, return FetchSuccess

**Independent Test**: Provide a known valid URL (e.g., https://httpbin.org/html) and verify HTML content is returned in a FetchSuccess object

### Implementation for User Story 1

- [ ] T011 [US1] Implement URL parsing with url::Url::parse in `src-tauri/src/url_ingestion/validator.rs` - parse raw URL into components, return parsed URL or validation error
- [ ] T012 [US1] Implement protocol validation in `src-tauri/src/url_ingestion/validator.rs` - reject non-HTTP/HTTPS schemes with FetchError::Validation
- [ ] T013 [US1] Implement validate() public function in `src-tauri/src/url_ingestion/validator.rs` - orchestrate parsing and protocol checks, return Result<url::Url, FetchError>
- [ ] T014 [US1] Implement host normalization in `src-tauri/src/url_ingestion/normalizer.rs` - lowercase host, convert IDN to Punycode using idna::domain_to_ascii()
- [ ] T015 [US1] Implement port normalization in `src-tauri/src/url_ingestion/normalizer.rs` - remove default ports (80 for http, 443 for https)
- [ ] T016 [US1] Implement path normalization in `src-tauri/src/url_ingestion/normalizer.rs` - ensure leading slash, remove trailing slash (except root)
- [ ] T017 [US1] Implement normalize() public function in `src-tauri/src/url_ingestion/normalizer.rs` - combine all normalizations, return NormalizedUrl
- [ ] T018 [US1] Create HTTP client builder in `src-tauri/src/url_ingestion/fetcher.rs` - configure reqwest::Client with 30s timeout, max 5 redirects, "RecipeScraper/1.0" User-Agent
- [ ] T019 [US1] Implement fetch() async function in `src-tauri/src/url_ingestion/fetcher.rs` - make GET request, return response or network error
- [ ] T020 [US1] Implement Content-Type validation in `src-tauri/src/url_ingestion/fetcher.rs` - verify response starts with "text/html", return FetchError::ContentType otherwise
- [ ] T021 [US1] Implement response body reading with size limit in `src-tauri/src/url_ingestion/fetcher.rs` - stream body up to 10MB, return FetchError::Size if exceeded
- [ ] T022 [US1] Implement ingest_url() orchestration function in `src-tauri/src/url_ingestion/mod.rs` - validate → normalize → fetch → return FetchSuccess
- [ ] T023 [US1] Implement validate_url() standalone function in `src-tauri/src/url_ingestion/mod.rs` - validate → normalize → return NormalizedUrl (no fetch)

**Checkpoint**: User Story 1 complete - valid URLs can be fetched and return HTML content

---

## Phase 4: User Story 2 - Handle Invalid URLs (Priority: P2)

**Goal**: Reject malformed URLs with clear validation error messages

**Independent Test**: Provide invalid inputs ("", "example.com", "ftp://example.com") and verify FetchError with Validation variant is returned

### Implementation for User Story 2

- [ ] T024 [US2] Add empty/whitespace input check in `src-tauri/src/url_ingestion/validator.rs` - return FetchError::Validation with "No URL provided" message
- [ ] T025 [US2] Add missing scheme detection in `src-tauri/src/url_ingestion/validator.rs` - detect URLs like "example.com", return FetchError::Validation with "Missing scheme" message
- [ ] T026 [US2] Add URL syntax error handling in `src-tauri/src/url_ingestion/validator.rs` - catch url::ParseError, return FetchError::Validation with descriptive message
- [ ] T027 [US2] Add protocol rejection messages in `src-tauri/src/url_ingestion/validator.rs` - return FetchError::Validation for ftp://, file://, mailto:, etc. with "scheme not allowed" message

**Checkpoint**: User Story 2 complete - invalid URLs are rejected with appropriate error messages

---

## Phase 5: User Story 3 - Handle Network Failures (Priority: P3)

**Goal**: Return structured network and HTTP errors for fetch failures

**Independent Test**: Provide unreachable URLs (non-existent domain, 404 page) and verify FetchError with Network or Http variant is returned

### Implementation for User Story 3

- [ ] T028 [US3] Add DNS resolution error handling in `src-tauri/src/url_ingestion/fetcher.rs` - catch reqwest DNS errors, return FetchError::Network with "DNS resolution failed" message
- [ ] T029 [US3] Add timeout error handling in `src-tauri/src/url_ingestion/fetcher.rs` - catch reqwest::Error timeout, return FetchError::Network with "Request timed out" message
- [ ] T030 [US3] Add connection error handling in `src-tauri/src/url_ingestion/fetcher.rs` - catch connection refused/reset, return FetchError::Network with descriptive message
- [ ] T031 [US3] Add HTTP 4xx status handling in `src-tauri/src/url_ingestion/fetcher.rs` - check status.is_client_error(), return FetchError::Http with status_code and message (404 → "Page not found")
- [ ] T032 [US3] Add HTTP 5xx status handling in `src-tauri/src/url_ingestion/fetcher.rs` - check status.is_server_error(), return FetchError::Http with status_code and message (500 → "Internal server error")

**Checkpoint**: User Story 3 complete - network and HTTP errors return structured error information

---

## Phase 6: User Story 4 - Tauri Integration (Priority: P4)

**Goal**: Expose URL ingestion as async Tauri commands callable from frontend

**Independent Test**: Invoke Tauri commands from JavaScript and verify JSON responses match contract types

### Implementation for User Story 4

- [ ] T033 [US4] Create ingest_url Tauri command in `src-tauri/src/url_ingestion/commands.rs` - async fn with #[tauri::command] attribute, accept url: String, return Result<FetchSuccess, FetchError>
- [ ] T034 [US4] Create validate_url Tauri command in `src-tauri/src/url_ingestion/commands.rs` - async fn with #[tauri::command] attribute, accept url: String, return Result<NormalizedUrl, FetchError>
- [ ] T035 [US4] Create shared HTTP client state in `src-tauri/src/main.rs` - initialize reqwest::Client once, wrap in tauri::State for command injection
- [ ] T036 [US4] Register commands in `src-tauri/src/main.rs` - add ingest_url and validate_url to tauri::Builder::invoke_handler()
- [ ] T037 [US4] Update `src-tauri/tauri.conf.json` to allow HTTP/HTTPS requests in security permissions

**Checkpoint**: User Story 4 complete - Tauri commands are accessible from frontend JavaScript

---

## Phase 7: User Story 5 - Remove Python Code (Priority: P5)

**Goal**: Clean up Python code after Rust implementation is verified

**Independent Test**: Verify no .py files exist in src/ or tests/, and pyproject.toml is removed

### Implementation for User Story 5

- [ ] T038 [US5] Verify Rust implementation matches Python behavior - manually test all acceptance scenarios from spec.md
- [ ] T039 [US5] Delete Python module `src/url_ingestion/` directory and all contents
- [ ] T040 [US5] Delete Python tests `tests/` directory and all contents
- [ ] T041 [US5] Delete `pyproject.toml` Python project configuration
- [ ] T042 [US5] Update `.gitignore` to remove Python-specific patterns, add Rust/Tauri patterns (target/, Cargo.lock for libraries)
- [ ] T043 [US5] Update `CLAUDE.md` to remove Python sections, keep only Rust/Tauri documentation

**Checkpoint**: User Story 5 complete - repository contains only Rust/Tauri code

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [ ] T044 [P] Add module-level documentation to `src-tauri/src/url_ingestion/mod.rs` with usage examples
- [ ] T045 [P] Create Claude Code Rust skill file in `.claude/skills/rust-patterns.md` with error handling and Tauri patterns
- [ ] T046 Run `cargo clippy` and fix any warnings in `src-tauri/`
- [ ] T047 Run `cargo fmt` to ensure consistent formatting in `src-tauri/`
- [ ] T048 Verify quickstart.md examples work with actual Rust implementation
- [ ] T049 Update plan.md to mark tasks.md as complete

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - delivers MVP
- **User Story 2 (Phase 4)**: Depends on Foundational - can start after US1 validator exists
- **User Story 3 (Phase 5)**: Depends on Foundational - can start after US1 fetcher exists
- **User Story 4 (Phase 6)**: Depends on US1, US2, US3 - requires all core functionality
- **User Story 5 (Phase 7)**: Depends on US4 - requires verified Tauri integration
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Core happy path
- **User Story 2 (P2)**: Extends validator.rs from US1 - adds validation error paths
- **User Story 3 (P3)**: Extends fetcher.rs from US1 - adds network/HTTP error paths
- **User Story 4 (P4)**: Wraps US1-3 functions in Tauri commands
- **User Story 5 (P5)**: Cleanup after US4 verification - sequential only

### Within Each User Story

- Validator before normalizer (US1)
- Normalizer before fetcher (US1)
- Error handling extends existing functions (US2, US3)
- Commands wrap existing functions (US4)

### Parallel Opportunities

- T003 and T004 can run in parallel (Setup phase)
- T005-T009 are sequential within Phase 2 (model dependencies)
- T044 and T045 can run in parallel (Polish phase)
- US2 and US3 can theoretically start in parallel after US1 core functions exist

---

## Parallel Example: Setup Phase

```bash
# Launch Setup tasks in parallel:
Task: "Configure src-tauri/tauri.conf.json"
Task: "Create src-tauri/src/url_ingestion/mod.rs"
```

## Parallel Example: Foundational Phase

```bash
# Sequential - models have dependencies:
Task: "Create ErrorType enum"  # T005 - first
Task: "Create NormalizedUrl struct"  # T006 - needs ErrorType
Task: "Create FetchSuccess struct"  # T007 - needs NormalizedUrl
Task: "Create FetchError enum"  # T008 - needs ErrorType
Task: "Create FetchResult type alias"  # T009 - needs FetchSuccess, FetchError
Task: "Export all public types"  # T010 - can run after all models
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (all models)
3. Complete Phase 3: User Story 1 (validation, normalization, fetching)
4. **STOP and VALIDATE**: Test with real recipe URL using `cargo test` or manual invocation
5. Can ship as working MVP - fetches HTML from valid URLs

### Incremental Delivery

1. Complete Setup + Foundational → Models ready
2. Add User Story 1 → Valid URLs work → **MVP Complete**
3. Add User Story 2 → Invalid URLs get helpful errors
4. Add User Story 3 → Network failures get helpful errors
5. Add User Story 4 → Frontend can invoke commands
6. Add User Story 5 → Python code removed
7. Polish → Skill files, documentation validated

### Recommended Sequence

Since US2 and US3 extend functions created in US1, the recommended sequence is:

1. Setup → Foundational → US1 (MVP)
2. US2 (validation errors - extends validator.rs)
3. US3 (network/HTTP errors - extends fetcher.rs)
4. US4 (Tauri commands - wraps everything)
5. US5 (Python removal - cleanup)
6. Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable after completion
- Commit after each task or logical group
- Create PR at end of each phase
- No test tasks generated (not requested in spec)
- All structs use `#[derive(Debug, Clone, Serialize, Deserialize)]`
- FetchError uses `#[serde(tag = "error_type", rename_all = "snake_case")]` for frontend compatibility
