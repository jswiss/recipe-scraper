# Tasks: Recipe Structure Extraction

**Input**: Design documents from `/specs/003-recipe-extraction/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md, this is a Tauri/Rust project:
- **Backend**: `src-tauri/src/`
- **Module**: `src-tauri/src/recipe_extraction/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure

- [x] T001 Add `scraper = "0.17"` dependency to src-tauri/Cargo.toml
- [x] T002 Create recipe_extraction module directory at src-tauri/src/recipe_extraction/
- [x] T003 Create module root file at src-tauri/src/recipe_extraction/mod.rs with submodule declarations
- [x] T004 Register recipe_extraction module in src-tauri/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and utilities that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 [P] Implement ExtractedField<T> enum (Found/NotFound variants) in src-tauri/src/recipe_extraction/models.rs
- [x] T006 [P] Implement ExtractionSource enum (JsonLd, Microdata, AiFallback) in src-tauri/src/recipe_extraction/models.rs
- [x] T007 [P] Implement Ingredient struct with name, quantity, unit, raw_text in src-tauri/src/recipe_extraction/models.rs
- [x] T008 [P] Implement Instruction struct with step_number, text in src-tauri/src/recipe_extraction/models.rs
- [x] T009 [P] Implement NutritionInfo struct with all nutrition fields in src-tauri/src/recipe_extraction/models.rs
- [x] T010 Implement ExtractedRecipe struct with all fields using ExtractedField<T> in src-tauri/src/recipe_extraction/models.rs
- [x] T011 Implement ExtractionError enum with all variants in src-tauri/src/recipe_extraction/models.rs
- [x] T012 [P] Implement ISO 8601 duration parser (PT15M -> 15 minutes) in src-tauri/src/recipe_extraction/duration.rs
- [x] T013 Export all types from src-tauri/src/recipe_extraction/mod.rs

**Checkpoint**: Foundation ready - core types available for all extraction implementations

---

## Phase 3: User Story 1 - Extract Recipe from Structured Data (Priority: P1)

**Goal**: Extract recipe data from JSON-LD and Microdata when available (FR-001, FR-002)

**Independent Test**: Provide HTML with valid JSON-LD recipe schema and verify all fields are correctly extracted

### Implementation for User Story 1

#### JSON-LD Extraction

- [x] T014 [P] [US1] Create json_ld.rs module file at src-tauri/src/recipe_extraction/json_ld.rs
- [x] T015 [US1] Implement find_jsonld_scripts() to extract `<script type="application/ld+json">` content using scraper in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T016 [US1] Implement is_recipe_schema() to check @type contains "Recipe" (string or array) in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T017 [US1] Implement parse_recipe_from_jsonld() to map JSON-LD fields to ExtractedRecipe in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T018 [US1] Handle @graph mode (array of objects) in JSON-LD extraction in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T019 [US1] Implement extract_ingredients_from_jsonld() for recipeIngredient parsing in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T020 [US1] Implement extract_instructions_from_jsonld() for recipeInstructions (strings and HowToStep) in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T021 [US1] Implement extract_nutrition_from_jsonld() for NutritionInformation in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T022 [US1] Implement extract_from_jsonld() public entry point returning Result<ExtractedRecipe, ExtractionError> in src-tauri/src/recipe_extraction/json_ld.rs

#### Microdata Extraction

- [x] T023 [P] [US1] Create microdata.rs module file at src-tauri/src/recipe_extraction/microdata.rs
- [x] T024 [US1] Implement find_recipe_itemscope() to locate `[itemscope][itemtype*="schema.org/Recipe"]` using scraper in src-tauri/src/recipe_extraction/microdata.rs
- [x] T025 [US1] Implement extract_itemprop() helper to get value from itemprop elements (content attr or text) in src-tauri/src/recipe_extraction/microdata.rs
- [x] T026 [US1] Implement extract_ingredients_from_microdata() for `[itemprop="recipeIngredient"]` elements in src-tauri/src/recipe_extraction/microdata.rs
- [x] T027 [US1] Implement extract_instructions_from_microdata() for `[itemprop="recipeInstructions"]` elements in src-tauri/src/recipe_extraction/microdata.rs
- [x] T028 [US1] Implement extract_nutrition_from_microdata() for nested nutrition itemscope in src-tauri/src/recipe_extraction/microdata.rs
- [x] T029 [US1] Implement extract_from_microdata() public entry point returning Result<ExtractedRecipe, ExtractionError> in src-tauri/src/recipe_extraction/microdata.rs

#### Command Integration for US1

- [x] T030 [P] [US1] Create commands.rs module file at src-tauri/src/recipe_extraction/commands.rs
- [x] T031 [US1] Implement extract_recipe Tauri command with JSON-LD → Microdata fallback chain in src-tauri/src/recipe_extraction/commands.rs
- [x] T032 [US1] Register extract_recipe command in Tauri invoke_handler in src-tauri/src/lib.rs
- [x] T033 [US1] Add unit tests for JSON-LD extraction with sample HTML in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T034 [US1] Add unit tests for Microdata extraction with sample HTML in src-tauri/src/recipe_extraction/microdata.rs

**Checkpoint**: User Story 1 complete - recipes with JSON-LD or Microdata can be extracted

---

## Phase 4: User Story 2 - Extract Recipe from HTML Content (Priority: P2)

**Goal**: Use local AI model to extract recipe data when no structured data is present (FR-002a)

**Independent Test**: Provide HTML without structured data markup and verify the AI model extracts reasonable fields

**Dependency**: Requires llama-cpp-sys-3 crate for local LLM inference

### Implementation for User Story 2

#### AI Model Infrastructure

- [ ] T035 Add `llama-cpp-sys-3 = { version = "0.5", features = ["native"] }` dependency to src-tauri/Cargo.toml
- [ ] T036 [P] [US2] Create ai_fallback.rs module file at src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T037 [US2] Implement ModelStatus struct with downloaded, loaded, model_path, model_size_bytes, model_name in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T038 [US2] Implement get_model_path() to return path ~/.config/recipe-scraper/models/gemma-2-2b-q4.gguf in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T039 [US2] Implement check_model_downloaded() to verify model file exists and size in src-tauri/src/recipe_extraction/ai_fallback.rs

#### Model Download

- [ ] T040 [US2] Implement DownloadProgress struct with downloaded_bytes, total_bytes, speed, eta in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T041 [US2] Implement ModelDownloadError enum (Network, Disk, Verification variants) in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T042 [US2] Implement download_model() async function with progress events via Tauri emit in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T043 [US2] Implement check_ai_model_status Tauri command in src-tauri/src/recipe_extraction/commands.rs
- [ ] T044 [US2] Implement download_ai_model Tauri command with progress events in src-tauri/src/recipe_extraction/commands.rs

#### AI Extraction

- [ ] T045 [US2] Implement load_model() to load GGUF model into memory using llama-cpp-sys-3 in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T046 [US2] Implement recipe extraction prompt template for HTML → JSON recipe in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T047 [US2] Implement run_inference() to generate recipe JSON from HTML using model in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T048 [US2] Implement parse_ai_response() to convert AI output to ExtractedRecipe in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T049 [US2] Implement extract_from_ai() public entry point with async spawn_blocking in src-tauri/src/recipe_extraction/ai_fallback.rs

#### Integration

- [ ] T050 [US2] Add model state Arc<Mutex<Option<LlamaModel>>> to Tauri managed state in src-tauri/src/lib.rs
- [ ] T051 [US2] Update extract_recipe command to fall back to AI extraction when JSON-LD/Microdata fail in src-tauri/src/recipe_extraction/commands.rs
- [ ] T052 [US2] Register check_ai_model_status and download_ai_model commands in src-tauri/src/lib.rs

**Checkpoint**: User Story 2 complete - recipes can be extracted from HTML without structured data using local AI

---

## Phase 5: User Story 3 - Handle Incomplete or Missing Data (Priority: P3)

**Goal**: Provide clear feedback when recipe data is incomplete with null justifications (FR-011, FR-012)

**Independent Test**: Provide HTML with partial recipe information and verify null fields include appropriate justifications

### Implementation for User Story 3

- [x] T053 [US3] Add standard justification messages as constants (NOT_FOUND, AMBIGUOUS, NOT_PROVIDED) in src-tauri/src/recipe_extraction/models.rs
- [ ] T054 [US3] Implement validate_recipe() to ensure all ExtractedField variants have justifications when NotFound in src-tauri/src/recipe_extraction/models.rs
- [x] T055 [US3] Update JSON-LD extractor to provide specific justifications for missing fields in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T056 [US3] Update Microdata extractor to provide specific justifications for missing fields in src-tauri/src/recipe_extraction/microdata.rs
- [ ] T057 [US3] Update AI extractor to provide justifications from model output in src-tauri/src/recipe_extraction/ai_fallback.rs
- [x] T058 [US3] Implement detect_no_recipe() to return NoRecipeFound error with html_preview in src-tauri/src/recipe_extraction/commands.rs
- [ ] T059 [US3] Add justification tests for partial data scenarios in src-tauri/src/recipe_extraction/json_ld.rs

**Checkpoint**: User Story 3 complete - all null fields include human-readable justifications

---

## Phase 6: User Story 4 - Extract Images (Priority: P4)

**Goal**: Extract recipe image URLs when available (FR-009)

**Independent Test**: Provide recipe HTML with images and verify image URLs are extracted as absolute URLs

### Implementation for User Story 4

- [x] T060 [US4] Implement extract_images_from_jsonld() for image field (string, array, or ImageObject) in src-tauri/src/recipe_extraction/json_ld.rs
- [ ] T061 [US4] Implement normalize_image_url() to convert relative URLs to absolute in src-tauri/src/recipe_extraction/json_ld.rs
- [x] T062 [US4] Implement extract_images_from_microdata() for `[itemprop="image"]` elements in src-tauri/src/recipe_extraction/microdata.rs
- [ ] T063 [US4] Update AI prompt to include image extraction from HTML img tags in src-tauri/src/recipe_extraction/ai_fallback.rs
- [ ] T064 [US4] Add image extraction tests for various image formats (string, array, object) in src-tauri/src/recipe_extraction/json_ld.rs

**Checkpoint**: User Story 4 complete - recipe images are extracted as absolute URLs

---

## Phase 7: Convenience Features & Integration

**Purpose**: Additional commands and cross-story integration

- [ ] T065 Implement ExtractFromUrlError enum wrapping FetchError and ExtractionError in src-tauri/src/recipe_extraction/models.rs
- [ ] T066 Implement extract_recipe_from_url Tauri command combining fetch and extract in src-tauri/src/recipe_extraction/commands.rs
- [ ] T067 Register extract_recipe_from_url command in src-tauri/src/lib.rs
- [ ] T068 Add integration test with real recipe URL fetching in src-tauri/src/recipe_extraction/commands.rs

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [x] T069 Run cargo clippy and fix all warnings in src-tauri/
- [x] T070 Run cargo fmt to ensure consistent formatting in src-tauri/
- [x] T071 Run cargo test to verify all tests pass
- [x] T072 Verify extract_recipe command works with sample JSON-LD HTML
- [x] T073 Verify extract_recipe command works with sample Microdata HTML
- [x] T074 Verify extract_recipe command returns proper error for non-recipe HTML
- [ ] T075 Update quickstart.md with actual test commands and results in specs/003-recipe-extraction/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - JSON-LD and Microdata extraction
- **User Story 2 (Phase 4)**: Depends on Foundational - AI fallback (can parallel with US1)
- **User Story 3 (Phase 5)**: Depends on US1 and US2 - adds justifications to existing extractors
- **User Story 4 (Phase 6)**: Depends on US1 and US2 - adds image extraction to existing extractors
- **Convenience (Phase 7)**: Depends on US1-US4 complete
- **Polish (Phase 8)**: Depends on all phases complete

### User Story Dependencies

- **User Story 1 (P1)**: Core extraction - no dependencies on other stories
- **User Story 2 (P2)**: AI fallback - can parallel with US1 after Foundational
- **User Story 3 (P3)**: Adds to US1+US2 extractors - requires both complete
- **User Story 4 (P4)**: Adds to US1+US2 extractors - requires both complete

### Within Each Phase

- Tasks marked [P] can run in parallel (different files)
- Non-[P] tasks have implicit dependencies on prior tasks in same phase
- Models before services, services before commands

### Parallel Opportunities

**Phase 2 (Foundational)**:
```
T005, T006, T007, T008, T009, T012 can all run in parallel
```

**Phase 3 (US1)** - After T013 completes:
```
T014 and T023 can run in parallel (json_ld.rs and microdata.rs)
T030 can run in parallel with JSON-LD and Microdata work
```

**Phase 4 (US2)** - Can run in parallel with Phase 3:
```
T036 can start as soon as T035 completes
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T013)
3. Complete Phase 3: User Story 1 - JSON-LD and Microdata (T014-T034)
4. **STOP and VALIDATE**: Test with real recipe URLs that have JSON-LD
5. Deploy/demo MVP - covers 95%+ of modern recipe sites

### Incremental Delivery

1. Setup + Foundational → Core types ready
2. Add User Story 1 → JSON-LD + Microdata extraction → MVP ready
3. Add User Story 2 → AI fallback for legacy sites → Broader coverage
4. Add User Story 3 → Better error messages → Improved UX
5. Add User Story 4 → Image extraction → Complete feature

### Suggested Scope

- **MVP**: Phases 1-3 (Setup + Foundational + US1) = 34 tasks
- **Full Feature**: All phases = 75 tasks

---

## Notes

- [P] tasks = different files, no dependencies
- [US#] label maps task to specific user story
- Commit after each task or logical group
- US2 (AI fallback) can be deferred if model setup is complex
- All null fields must have justifications per FR-012

---

## Progress Summary

**MVP Complete**: Phases 1-3 (34 tasks)

| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| Phase 1: Setup | 4 | 4 | DONE |
| Phase 2: Foundational | 9 | 9 | DONE |
| Phase 3: User Story 1 | 21 | 21 | DONE |
| Phase 4: User Story 2 | 18 | 0 | Pending |
| Phase 5: User Story 3 | 7 | 4 | Partial |
| Phase 6: User Story 4 | 5 | 2 | Partial |
| Phase 7: Convenience | 4 | 0 | Pending |
| Phase 8: Polish | 7 | 6 | Partial |

**Tests Passing**: 53 tests (31 new for recipe_extraction + 22 existing)
