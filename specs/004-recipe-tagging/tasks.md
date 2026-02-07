# Tasks: Recipe Tagging and Categorization

**Input**: Design documents from `/specs/004-recipe-tagging/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included inline per user story (acceptance scenarios require verification).

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create module skeleton and shared types

- [ ] T001 Create `src-tauri/src/recipe_tagging/` directory, `mod.rs` with module declarations (models, vocabulary, scoring, cuisine_tagger, course_tagger, diet_tagger, commands), and `models.rs` with all types from data-model.md: `Tag`, `TagDomain`, `TagSet` (with `empty()` and sort/filter methods), `TaggingResult`, `TaggingError` (with serde tagged enum), and `DietaryFlag` enum. Include unit tests for Tag creation, TagSet::empty(), confidence validation, and serde round-trip serialization.
- [ ] T002 [P] Create `src-tauri/src/recipe_tagging/scoring.rs` with shared confidence helpers: `normalize_score(raw: f64, max: f64) -> f64` (clamps to [0.0, 1.0]), `filter_by_threshold(tags: Vec<Tag>, threshold: f64) -> Vec<Tag>` (removes below threshold, default 0.5 per FR-006), `sort_by_confidence(tags: &mut Vec<Tag>)` (descending per FR-007). Include unit tests.
- [ ] T003 Register `pub mod recipe_tagging;` in `src-tauri/src/lib.rs` (module declaration only, no commands in invoke_handler yet). Verify `cargo build` succeeds.

**Checkpoint**: Module compiles with empty tagger files. `cargo build` passes.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Vocabulary data that ALL tagging domains depend on

**CRITICAL**: No user story work can begin until vocabulary data exists.

- [ ] T004 Create `src-tauri/src/recipe_tagging/vocabulary.rs` with struct definitions (`CuisineEntry`, `CourseEntry`, `DietDefinition` per data-model.md) and populate cuisine vocabulary: ~35 `CuisineEntry` items covering major world cuisines (Italian, Mexican, Japanese, Chinese, Thai, Indian, French, Greek, Mediterranean, Korean, Vietnamese, American, Southern, Cajun/Creole, Caribbean, Middle Eastern, Ethiopian, Moroccan, Turkish, Spanish, Portuguese, German, British, Irish, Scandinavian, Russian, Polish, Brazilian, Peruvian, Filipino, Indonesian, Malaysian, Hawaiian, Tex-Mex) each with `title_keywords`, `ingredient_keywords`, and `instruction_keywords`. Expose via `pub fn cuisine_vocabulary() -> &'static [CuisineEntry]`.
- [ ] T005 Add course vocabulary (~12 `CourseEntry` items: breakfast, brunch, lunch, dinner, appetizer, side dish, main course, dessert, snack, beverage, soup, salad) with `title_keywords`, `ingredient_keywords`, `contextual_keywords` to `src-tauri/src/recipe_tagging/vocabulary.rs`. Expose via `pub fn course_vocabulary() -> &'static [CourseEntry]`.
- [ ] T006 Add diet vocabulary (~15 `DietDefinition` items: vegan, vegetarian, pescatarian, gluten-free, dairy-free, nut-free, egg-free, soy-free, keto, paleo, whole30, low-carb, low-fat, sugar-free, Mediterranean diet) with `excluded_ingredients` and `excluded_categories`, plus ingredient synonym/alias map (~200-300 entries mapping variations like "AP flour" → "wheat flour") and ingredient dietary properties map (`HashMap<&str, &[DietaryFlag]>`) to `src-tauri/src/recipe_tagging/vocabulary.rs`. Expose via `pub fn diet_vocabulary()`, `pub fn ingredient_aliases()`, `pub fn ingredient_properties()`.
- [ ] T007 Update `src-tauri/src/recipe_tagging/mod.rs` to export all foundational modules (`pub mod vocabulary`, `pub mod scoring`, `pub mod models`) and re-export key public types (`Tag`, `TagSet`, `TaggingResult`, `TaggingError`). Verify `cargo build` passes.

**Checkpoint**: All vocabulary data compiled and accessible. `cargo test` passes for models and scoring.

---

## Phase 3: User Story 1 - Automatic Cuisine Tagging (Priority: P1) MVP

**Goal**: Assign cuisine tags (e.g., Italian, Thai, Mexican) to a recipe based on title, ingredients, and instructions with confidence scores.

**Independent Test**: Provide an ExtractedRecipe with Thai indicators (title "Pad Thai", ingredients [fish sauce, rice noodles]) and verify "Thai" tag with confidence > 0.7.

### Implementation for User Story 1

- [ ] T008 [US1] Create `src-tauri/src/recipe_tagging/cuisine_tagger.rs` implementing `pub fn tag(recipe: &ExtractedRecipe) -> Vec<Tag>`. Logic per research.md R2: for each CuisineEntry in vocabulary, compute `confidence = 0.35 * title_signal + 0.35 * ingredient_signal + 0.30 * instruction_signal`. Title signal = 1.0 if any title_keyword found in lowercased recipe title, 0.0 otherwise. Ingredient signal = min(matched_ingredient_keywords / 2, 1.0). Instruction signal = min(matched_instruction_keywords / 1, 1.0). Use `filter_by_threshold` and `sort_by_confidence` from scoring.rs. Handle `ExtractedField::NotFound` gracefully (signal = 0.0 for missing fields).
- [ ] T009 [US1] Add unit tests for cuisine tagger in `src-tauri/src/recipe_tagging/cuisine_tagger.rs`: (1) Thai recipe with clear indicators → "Thai" tag with confidence > 0.7, (2) recipe with pasta + Italian herbs → multiple cuisine tags with "Italian" highest, (3) recipe with soy sauce + rice noodles → multiple Asian cuisine tags, (4) recipe with no recognizable indicators → empty cuisine tags, (5) recipe with missing title → still tags from ingredients. Verify `cargo test recipe_tagging::cuisine_tagger` passes.

**Checkpoint**: Cuisine tagging works independently. US1 acceptance scenarios verified.

---

## Phase 4: User Story 2 - Automatic Course Tagging (Priority: P2)

**Goal**: Assign course tags (e.g., breakfast, dessert, appetizer) based on title, description, ingredients, and contextual cues.

**Independent Test**: Provide an ExtractedRecipe titled "Blueberry Pancakes" with flour/eggs/blueberries and verify "breakfast" tag with confidence > 0.7.

### Implementation for User Story 2

- [ ] T010 [US2] Create `src-tauri/src/recipe_tagging/course_tagger.rs` implementing `pub fn tag(recipe: &ExtractedRecipe) -> Vec<Tag>`. Logic per research.md R2: for each CourseEntry, compute `confidence = 0.40 * title_signal + 0.25 * ingredient_signal + 0.20 * description_signal + 0.15 * contextual_signal`. Title signal checks title_keywords in lowercased title. Ingredient signal checks ingredient_keywords against ingredient names. Description signal checks title_keywords in description. Contextual signal checks contextual_keywords across all text fields. Use scoring helpers. Handle missing fields gracefully.
- [ ] T011 [US2] Add unit tests for course tagger in `src-tauri/src/recipe_tagging/course_tagger.rs`: (1) "Blueberry Pancakes" → "breakfast" > 0.7, (2) "Chocolate Cake" with frosting → "dessert" highest, (3) salad recipe → multiple course tags (appetizer, side dish, lunch), (4) "Appetizer: Bruschetta" with explicit course in title → stated course highest confidence, (5) recipe with no course indicators → empty. Verify `cargo test recipe_tagging::course_tagger` passes.

**Checkpoint**: Course tagging works independently. US2 acceptance scenarios verified.

---

## Phase 5: User Story 3 - Automatic Diet Tagging (Priority: P3)

**Goal**: Assign dietary tags (vegan, gluten-free, etc.) based on ingredient analysis using normalized matching with synonym/alias map.

**Independent Test**: Provide a recipe with only plant-based ingredients and verify "vegan" and "vegetarian" tags with high confidence. Provide a recipe with wheat flour and verify NO "gluten-free" tag (SC-006).

### Implementation for User Story 3

- [ ] T012 [US3] Create `src-tauri/src/recipe_tagging/diet_tagger.rs` implementing `pub fn tag(recipe: &ExtractedRecipe) -> Vec<Tag>`. Include `fn normalize_ingredient(name: &str) -> String` that lowercases, strips common prefixes ("organic ", "fresh ", "dried ", "ground ", "chopped "), and looks up in ingredient_aliases() map. For each DietDefinition: start confidence at 1.0, for each ingredient check if canonical name or its DietaryFlags violate the diet (confidence → 0.0 if violated, reduce by 0.2 if ambiguous). If recipe has no ingredients → confidence = 0.0 for ALL diet tags (fail-safe per SC-006). Use scoring helpers.
- [ ] T013 [US3] Add unit tests for diet tagger in `src-tauri/src/recipe_tagging/diet_tagger.rs`: (1) all plant-based ingredients → "vegan" and "vegetarian" with high confidence, (2) recipe with wheat flour → NO "gluten-free" tag (SC-006 critical), (3) recipe with "chicken breast" → no vegan/vegetarian tags, (4) recipe with "butter" (ambiguous) → reduced confidence on dairy-free, (5) recipe with no ingredients → empty diet tags (fail-safe), (6) ingredient normalization: "organic all-purpose flour" → "wheat flour" → gluten flag. Verify `cargo test recipe_tagging::diet_tagger` passes.

**Checkpoint**: Diet tagging works independently. SC-006 (zero false-positive safety-critical diet tags) verified.

---

## Phase 6: User Story 4 - View Tags with Confidence / Tauri Commands (Priority: P4)

**Goal**: Expose tagging through Tauri commands with structured output grouped by domain, ordered by confidence. Enable auto-tag after extraction and on-demand re-tagging.

**Independent Test**: Call `tag_recipe` with an ExtractedRecipe and verify response has cuisine/course/diet arrays sorted by confidence descending.

### Implementation for User Story 4

- [ ] T014 [US4] Implement `pub fn tag_recipe_from_extracted(recipe: &ExtractedRecipe, refine: bool) -> TagSet` orchestrator in `src-tauri/src/recipe_tagging/mod.rs` (or a dedicated file). Calls `cuisine_tagger::tag()`, `course_tagger::tag()`, `diet_tagger::tag()`, assembles TagSet. When `refine = true`, defer to heuristic path (stub returning default tags for now, implemented in Phase 7). Export from mod.rs.
- [ ] T015 [US4] Create `src-tauri/src/recipe_tagging/commands.rs` with two Tauri commands per contracts/tauri-commands.md: (1) `tag_recipe(recipe: ExtractedRecipe, refine: Option<bool>) -> Result<TagSet, TaggingError>` wrapping `tag_recipe_from_extracted`, (2) `extract_and_tag(html: String) -> Result<TaggingResult, TaggingError>` calling the public `recipe_extraction::extract_recipe` async fn then `tag_recipe_from_extracted`, mapping ExtractionError to TaggingError::ExtractionFailed. Do NOT modify the recipe_extraction module.
- [ ] T016 [US4] Register `tag_recipe` and `extract_and_tag` commands in `src-tauri/src/lib.rs` invoke_handler array. Update mod.rs to export commands. Verify `cargo build` succeeds and commands are callable.
- [ ] T017 [US4] Add unit tests in `src-tauri/src/recipe_tagging/commands.rs`: (1) tag_recipe with valid recipe returns TagSet with all three domains, (2) tag_recipe with empty recipe returns empty TagSet (not error), (3) verify tags are sorted by confidence descending within each domain, (4) verify no tag below 0.5 threshold appears in output. Verify `cargo test recipe_tagging::commands` passes.

**Checkpoint**: Full tagging pipeline callable via Tauri IPC. US4 acceptance scenarios verified (tags grouped by domain, ordered by confidence).

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Heuristic refinement, edge cases, final validation

- [ ] T018 Implement heuristic refinement mode in `src-tauri/src/recipe_tagging/scoring.rs` (FR-016): add `pub fn refine_scores(cuisine: &mut Vec<Tag>, course: &mut Vec<Tag>, recipe: &ExtractedRecipe)` that applies co-occurrence bonuses (1.2x multiplier when same tag has signals in title + ingredients) and cross-domain signals (e.g., "dim sum" cuisine boosts "appetizer" course). Re-normalize scores to [0.0, 1.0]. Wire into `tag_recipe_from_extracted` when `refine = true`. Add unit tests.
- [ ] T019 Add edge case tests in `src-tauri/src/recipe_tagging/` covering: (1) completely empty ExtractedRecipe (all NotFound, no ingredients) → empty TagSet, (2) recipe with only a title and no other fields → cuisine/course may tag from title alone, (3) fusion recipe with indicators from multiple cuisines → multiple cuisine tags with distributed confidence, (4) all confidence scores below 0.5 → empty domain, (5) recipe with non-English ingredient names (e.g., "dashi", "gochujang") → tags from recognized keywords with reduced confidence for unrecognized terms. Verify all edge cases from spec.md are covered.
- [ ] T020 Run full validation: `cargo test` (all tests pass), `cargo clippy` (no warnings), `cargo fmt --check` (formatted). Verify SC-001 (<1 second) by timing a tagging call in a test.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (T001-T003 complete)
- **User Stories (Phase 3-6)**: All depend on Phase 2 (vocabulary data ready)
  - US1, US2, US3 can proceed in parallel (different files, independent domains)
  - US4 depends on US1 + US2 + US3 (orchestrator calls all three taggers)
- **Polish (Phase 7)**: Depends on Phase 6 (all commands wired)

### User Story Dependencies

- **US1 (P1 - Cuisine)**: After Phase 2 — no dependencies on other stories
- **US2 (P2 - Course)**: After Phase 2 — no dependencies on other stories
- **US3 (P3 - Diet)**: After Phase 2 — no dependencies on other stories
- **US4 (P4 - Commands)**: After US1 + US2 + US3 — orchestrator needs all taggers

### Within Each User Story

- Tagger implementation before tests (tests call the tagger)
- Core logic before edge cases

### Parallel Opportunities

- T001 and T002 can run in parallel (different files: models.rs vs scoring.rs)
- T004, T005, T006 are sequential (same file: vocabulary.rs)
- US1 (T008-T009), US2 (T010-T011), US3 (T012-T013) can all run in parallel after Phase 2
- T018, T019 can run in parallel (different concerns: scoring.rs vs test files)

---

## Parallel Example: User Stories 1-3

```bash
# After Phase 2 completes, launch all three domain taggers in parallel:
Task: "T008 [US1] Implement cuisine_tagger.rs"
Task: "T010 [US2] Implement course_tagger.rs"
Task: "T012 [US3] Implement diet_tagger.rs"

# Then their tests in parallel:
Task: "T009 [US1] Add cuisine tagger tests"
Task: "T011 [US2] Add course tagger tests"
Task: "T013 [US3] Add diet tagger tests"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T007)
3. Complete Phase 3: US1 Cuisine Tagging (T008-T009)
4. **STOP and VALIDATE**: Test cuisine tagging independently
5. Proceed to remaining stories

### Incremental Delivery

1. Setup + Foundational → Module compiles
2. Add US1 (Cuisine) → Test independently (MVP!)
3. Add US2 (Course) → Test independently
4. Add US3 (Diet) → Test independently, verify SC-006
5. Add US4 (Commands) → Full pipeline via Tauri IPC
6. Polish → Heuristic refinement, edge cases, final validation

### Parallel Team Strategy

After Phase 2 completes:
- Developer A: US1 (Cuisine tagger)
- Developer B: US2 (Course tagger)
- Developer C: US3 (Diet tagger)
- Then any developer: US4 (Commands — integrates all three)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- SC-006 (zero false-positive diet tags) is the highest-risk acceptance criterion — test exhaustively in US3
