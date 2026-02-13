# Tasks: Robots.txt Compliance

**Input**: Design documents from `/specs/006-robots-txt-compliance/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependency and create module skeleton

- [x] T001 Add `robotstxt = "0.3.0"` dependency to `src-tauri/Cargo.toml` and run `cargo build` to verify compilation
- [x] T002 Create `robots_compliance` module skeleton with `src-tauri/src/robots_compliance/mod.rs` (pub mod declarations for models, checker, crawl_delay, commands) and declare `pub mod robots_compliance;` in `src-tauri/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Models, error types, and database migration that ALL user stories depend on

- [x] T003 [P] Define `RobotsDecision`, `CacheSource`, and `RobotsError` types with serde derives in `src-tauri/src/robots_compliance/models.rs` per data-model.md
- [x] T004 [P] Create SQLite migration `src-tauri/src/storage/migrations/002_robots_cache.sql` with `robots_cache` table (domain TEXT PK, raw_content TEXT, fetched_at TEXT, status TEXT with CHECK constraint) and update `Database::run_migrations()` in `src-tauri/src/storage/database.rs` to apply it

**Checkpoint**: Models compile, migration runs on app startup, module skeleton wires up

---

## Phase 3: User Story 1 — Check If Scraping Is Allowed (Priority: P1) MVP

**Goal**: Given a URL, fetch its site's robots.txt, parse it, and return an allowed/disallowed decision. Expose as a standalone Tauri command and integrate as a gate in `ingest_url`.

**Independent Test**: Submit a URL whose robots.txt disallows the path → system returns "disallowed" decision without fetching page content.

### Implementation for User Story 1

- [x] T005 [US1] Implement `fetch_robots_txt()` in `src-tauri/src/robots_compliance/checker.rs` — fetches `https://{domain}/robots.txt` using the shared `HttpClient`, enforces 10s timeout and 500KB max size, returns raw content or status (not_found, unreachable, oversized)
- [x] T006 [US1] Implement `get_or_fetch_robots()` cache layer in `src-tauri/src/robots_compliance/checker.rs` — checks `robots_cache` table for a valid entry (within 24h TTL), fetches and caches on miss/expiry, returns raw content + CacheSource
- [x] T007 [US1] Implement `check_compliance()` core logic in `src-tauri/src/robots_compliance/checker.rs` — uses `robotstxt::DefaultMatcher::one_agent_allowed_by_robots()` with a `USER_AGENT` constant (default: "RecipeScraper", easily configurable) to evaluate URL against cached/fetched robots.txt, builds and returns `RobotsDecision` with allowed status, reason, and matched_agent (satisfies FR-003 via crate and FR-007 via constant)
- [x] T008 [US1] Implement `check_robots_compliance` Tauri command in `src-tauri/src/robots_compliance/commands.rs` per contract — accepts `url: String`, `client: State<HttpClient>`, `db: State<Database>`, delegates to `check_compliance()`
- [x] T009 [US1] Register `check_robots_compliance` command in `src-tauri/src/lib.rs` invoke_handler and add necessary imports
- [x] T010 [US1] Add `RobotsDisallowed { message, url, reason }` variant to `FetchError` in `src-tauri/src/url_ingestion/models.rs`
- [x] T011 [US1] Integrate compliance gate into `ingest_url` in `src-tauri/src/url_ingestion/commands.rs` — call `check_compliance()` after URL validation/normalization, return `FetchError::RobotsDisallowed` if disallowed, pass `State<Database>` parameter
- [x] T012 [US1] Run `cargo test` and `cargo clippy` to verify US1 compiles and passes all existing tests

**Checkpoint**: `check_robots_compliance` command works standalone; `ingest_url` blocks disallowed URLs. MVP is functional.

---

## Phase 4: User Story 2 — Respect Crawl Delay (Priority: P2)

**Goal**: Extract `Crawl-delay` directive from robots.txt and include it in the decision object.

**Independent Test**: Check a robots.txt with `Crawl-delay: 10` → decision object contains `crawl_delay_secs: 10.0`.

### Implementation for User Story 2

- [ ] T013 [US2] Implement `parse_crawl_delay()` in `src-tauri/src/robots_compliance/crawl_delay.rs` — parses raw robots.txt content, finds the `Crawl-delay` value for the matched user-agent group (case-insensitive), returns `Option<f64>` in seconds
- [ ] T014 [US2] Integrate `parse_crawl_delay()` into `check_compliance()` in `src-tauri/src/robots_compliance/checker.rs` — call after allowed/disallowed check, populate `crawl_delay_secs` field in `RobotsDecision`
- [ ] T015 [US2] Run `cargo test` and `cargo clippy` to verify US2 passes

**Checkpoint**: Decision objects now include crawl delay when present. No crawl delay returns `None`.

---

## Phase 5: User Story 3 — Handle Missing or Unreachable robots.txt (Priority: P3)

**Goal**: Correctly handle all edge cases for robots.txt availability per RFC 9309.

**Independent Test**: URL with 404 robots.txt returns "allowed"; URL with timeout returns "disallowed" with reason.

### Implementation for User Story 3

- [ ] T016 [US3] Refine HTTP status handling in `fetch_robots_txt()` in `src-tauri/src/robots_compliance/checker.rs` — ensure 404/410 returns status "not_found", 5xx returns "unreachable", empty body returns "ok" with empty content, oversized (>500KB) returns "oversized"
- [ ] T017 [US3] Map fetch statuses to correct decisions in `check_compliance()` in `src-tauri/src/robots_compliance/checker.rs` — "not_found" → allowed (RFC 9309), "unreachable" → disallowed, "oversized" → allowed, empty content → allowed, with appropriate reason strings per contracts
- [ ] T018 [US3] Implement stale cache fallback in `get_or_fetch_robots()` in `src-tauri/src/robots_compliance/checker.rs` — if cache is expired and re-fetch fails, use stale cached entry with reason suffix "(stale cache)" and `source: Cached`
- [ ] T019 [US3] Run `cargo test` and `cargo clippy` to verify US3 edge cases

**Checkpoint**: All robots.txt availability scenarios handled correctly. Offline users with cached data get decisions.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Cleanup, startup maintenance, and final validation

- [ ] T020 Add startup cache cleanup in `Database::new()` or setup closure in `src-tauri/src/lib.rs` — prune `robots_cache` entries older than 7 days
- [ ] T021 Run `cargo fmt`, `cargo clippy`, and full `cargo test` suite to validate all phases

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — delivers MVP
- **US2 (Phase 4)**: Depends on US1 (extends checker.rs and RobotsDecision)
- **US3 (Phase 5)**: Depends on US1 (refines checker.rs edge cases)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — no dependencies on other stories
- **US2 (P2)**: Depends on US1 (adds crawl_delay field to existing checker flow)
- **US3 (P3)**: Depends on US1 (refines fetch/status handling in existing checker)
- **US2 and US3**: Could run in parallel after US1 (different concerns in checker.rs, but touch same file — sequential recommended)

### Within Each User Story

- Models before services
- Core logic before Tauri command wrapper
- Standalone command before `ingest_url` integration
- Validation (clippy/test) at end of each phase

### Parallel Opportunities

- T003 and T004 can run in parallel (different files: models.rs vs migration SQL + database.rs)
- T010 can run in parallel with T008/T009 (different module: url_ingestion vs robots_compliance)

---

## Parallel Example: Phase 2

```
# These can run in parallel (different files):
Task T003: "Define models in src-tauri/src/robots_compliance/models.rs"
Task T004: "Create migration in src-tauri/src/storage/migrations/002_robots_cache.sql"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T004)
3. Complete Phase 3: User Story 1 (T005-T012)
4. **STOP and VALIDATE**: `check_robots_compliance` works standalone, `ingest_url` gates on compliance
5. This is a deployable MVP

### Incremental Delivery

1. Setup + Foundational → Module ready
2. Add US1 → Core compliance checking works → MVP
3. Add US2 → Crawl delay surfaced in decisions
4. Add US3 → All edge cases handled, offline support complete
5. Polish → Cache cleanup, final validation
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- The `robotstxt` crate handles Allow/Disallow parsing; custom `crawl_delay.rs` handles Crawl-delay only
