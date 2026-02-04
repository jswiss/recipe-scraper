# Tasks: URL Ingestion

**Input**: Design documents from `/specs/001-url-ingestion/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in the feature specification. Test tasks omitted.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Python module: `src/url_ingestion/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create project directory structure: `src/url_ingestion/` and `tests/unit/`, `tests/integration/`
- [X] T002 Initialize Python project with pyproject.toml including requests>=2.28 dependency
- [X] T003 [P] Create `src/url_ingestion/__init__.py` with module exports (ingest_url, validate_url, FetchSuccess, FetchError, ErrorType, NormalizedURL)
- [X] T004 [P] Configure ruff for linting in pyproject.toml

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data models and error types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 Create ErrorType enum in `src/url_ingestion/models.py` with values: VALIDATION, NETWORK, HTTP, CONTENT_TYPE, SIZE
- [X] T006 Create NormalizedURL dataclass in `src/url_ingestion/models.py` with fields: scheme, host, port, path, query, fragment, and url property
- [X] T007 Create FetchSuccess dataclass in `src/url_ingestion/models.py` with fields: url, html, status_code, content_type, final_url
- [X] T008 Create FetchError dataclass in `src/url_ingestion/models.py` with fields: error_type, message, url, details
- [X] T009 Create FetchResult type alias (Union[FetchSuccess, FetchError]) in `src/url_ingestion/models.py`

**Checkpoint**: Foundation ready - all data models defined, user story implementation can begin

---

## Phase 3: User Story 1 - Fetch Valid Recipe URL (Priority: P1) 🎯 MVP

**Goal**: Accept valid URLs, normalize them, fetch HTML content, return FetchSuccess

**Independent Test**: Provide a known valid URL (e.g., https://httpbin.org/html) and verify HTML content is returned in a FetchSuccess object

### Implementation for User Story 1

- [X] T010 [US1] Implement URL parsing with urllib.parse.urlparse in `src/url_ingestion/validator.py` - parse raw URL into components
- [X] T011 [US1] Implement protocol validation in `src/url_ingestion/validator.py` - reject non-HTTP/HTTPS schemes
- [X] T012 [US1] Implement URL normalization in `src/url_ingestion/normalizer.py` - lowercase scheme/host, remove default ports (80/443), remove trailing slashes
- [X] T013 [US1] Implement IDN to Punycode conversion in `src/url_ingestion/normalizer.py` using idna.encode() for international domains
- [X] T014 [US1] Implement percent-encoding normalization in `src/url_ingestion/normalizer.py` - decode unnecessarily encoded chars
- [X] T015 [US1] Implement HTTP fetcher in `src/url_ingestion/fetcher.py` with requests.get(), 30-second timeout, max 5 redirects
- [X] T016 [US1] Add User-Agent header ("RecipeScraper/1.0") to requests in `src/url_ingestion/fetcher.py`
- [X] T017 [US1] Implement Content-Type validation in `src/url_ingestion/fetcher.py` - verify response is text/html
- [X] T018 [US1] Implement response size limiting in `src/url_ingestion/fetcher.py` using iter_content() with 10MB max
- [X] T019 [US1] Implement ingest_url() main entry point in `src/url_ingestion/__init__.py` - orchestrate validate→normalize→fetch→return FetchSuccess

**Checkpoint**: User Story 1 complete - valid URLs can be fetched and return HTML content

---

## Phase 4: User Story 2 - Handle Invalid URLs (Priority: P2)

**Goal**: Reject malformed URLs with clear ValidationError messages

**Independent Test**: Provide invalid inputs ("not-a-url", "ftp://example.com", "", "example.com") and verify FetchError with error_type=VALIDATION is returned

### Implementation for User Story 2

- [X] T020 [US2] Add empty/whitespace input check in `src/url_ingestion/validator.py` - return FetchError(VALIDATION) with "No URL provided"
- [X] T021 [US2] Add URL syntax validation in `src/url_ingestion/validator.py` - return FetchError(VALIDATION) for malformed URLs
- [X] T022 [US2] Add missing protocol detection in `src/url_ingestion/validator.py` - return FetchError(VALIDATION) with "Missing scheme (http:// or https://)"
- [X] T023 [US2] Add protocol rejection for non-HTTP in `src/url_ingestion/validator.py` - return FetchError(VALIDATION) for ftp://, file://, etc.
- [X] T024 [US2] Implement validate_url() standalone function in `src/url_ingestion/__init__.py` for pre-validation without fetching

**Checkpoint**: User Story 2 complete - invalid URLs are rejected with appropriate error messages

---

## Phase 5: User Story 3 - Handle Unreachable URLs (Priority: P3)

**Goal**: Return structured NetworkError and HttpError for fetch failures

**Independent Test**: Provide unreachable URLs (non-existent domain, 404 page) and verify FetchError with error_type=NETWORK or HTTP is returned

### Implementation for User Story 3

- [X] T025 [US3] Add DNS resolution error handling in `src/url_ingestion/fetcher.py` - catch requests.exceptions.ConnectionError, return FetchError(NETWORK)
- [X] T026 [US3] Add timeout error handling in `src/url_ingestion/fetcher.py` - catch requests.exceptions.Timeout, return FetchError(NETWORK) with "Request timed out"
- [X] T027 [US3] Add connection refused handling in `src/url_ingestion/fetcher.py` - catch connection errors, return FetchError(NETWORK)
- [X] T028 [US3] Add HTTP 4xx status handling in `src/url_ingestion/fetcher.py` - return FetchError(HTTP) with status code in details
- [X] T029 [US3] Add HTTP 5xx status handling in `src/url_ingestion/fetcher.py` - return FetchError(HTTP) with status code in details
- [X] T030 [US3] Add Content-Type rejection in `src/url_ingestion/fetcher.py` - return FetchError(CONTENT_TYPE) for non-HTML responses
- [X] T031 [US3] Add size limit rejection in `src/url_ingestion/fetcher.py` - return FetchError(SIZE) when response exceeds 10MB

**Checkpoint**: All user stories complete - full error handling for validation, network, and HTTP errors

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [X] T032 [P] Update `src/url_ingestion/__init__.py` exports to include all public types
- [X] T033 [P] Add module docstring to `src/url_ingestion/__init__.py` with usage examples
- [X] T034 Run quickstart.md validation - execute example code snippets to verify they work
- [X] T035 Verify all error messages are human-readable and actionable

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - delivers MVP
- **User Story 2 (Phase 4)**: Depends on Foundational - can run parallel to US1
- **User Story 3 (Phase 5)**: Depends on Foundational - can run parallel to US1/US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Core happy path
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Adds validation error paths
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Adds network/HTTP error paths

### Within Each User Story

- Validator before normalizer (US1)
- Normalizer before fetcher (US1)
- Error handling extends existing functions (US2, US3)

### Parallel Opportunities

- T003 and T004 can run in parallel (Setup phase)
- T005-T009 are sequential (model dependencies)
- US1, US2, US3 can theoretically run in parallel after Foundational, but US1 creates the core functions that US2/US3 extend
- T032 and T033 can run in parallel (Polish phase)

---

## Parallel Example: Setup Phase

```bash
# Launch Setup tasks in parallel:
Task: "Create src/url_ingestion/__init__.py with module exports"
Task: "Configure ruff for linting in pyproject.toml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (all models)
3. Complete Phase 3: User Story 1 (validation, normalization, fetching)
4. **STOP and VALIDATE**: Test with real recipe URL
5. Can ship as working MVP - fetches HTML from valid URLs

### Incremental Delivery

1. Complete Setup + Foundational → Models ready
2. Add User Story 1 → Valid URLs work → **MVP Complete**
3. Add User Story 2 → Invalid URLs get helpful errors
4. Add User Story 3 → Network failures get helpful errors
5. Polish → Clean exports, docs validated

### Recommended Sequence

Since US2 and US3 add error handling to functions created in US1, the recommended sequence is:

1. Setup → Foundational → US1 (MVP)
2. US2 (validation errors)
3. US3 (network/HTTP errors)
4. Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable
- Commit after each task or logical group
- No test tasks generated (not requested in spec)
- Models are frozen dataclasses (immutable)
