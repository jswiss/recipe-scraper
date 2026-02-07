# Research: Recipe Tagging and Categorization

**Feature**: 004-recipe-tagging | **Date**: 2026-02-07

## R1: Rule-Based Tagging Strategy

**Decision**: Keyword/pattern matching with weighted signals per domain.

**Rationale**: Local-first constraint (no network) and <1s performance target rule out external APIs and large ML models. Rule-based matching with curated keyword lists is deterministic, auditable, fast, and requires zero additional dependencies. The existing `scraper` and `serde` crates are sufficient.

**Alternatives considered**:
- Embedded ML classifier (e.g., ONNX runtime): Adds ~20MB binary size, complex dependency, overkill for categorization of curated labels. Rejected per Constitution III (Minimal Dependencies).
- TF-IDF / statistical matching: Requires corpus training data and vectorization libraries. Over-engineered for predefined vocabularies. Better suited if tag vocabularies were open-ended.
- Regex-only matching: Too brittle for ingredient name variations. Substring matching with normalization is more resilient.

## R2: Confidence Score Computation

**Decision**: Weighted multi-signal scoring with domain-specific formulas.

**Rationale**: Different domains derive confidence from different recipe fields. A single formula would underfit. Each domain uses a weighted sum of signal hits normalized to [0.0, 1.0].

**Formulas**:

- **Cuisine**: `confidence = 0.35 * title_signal + 0.35 * ingredient_signal + 0.30 * instruction_signal`. Title signal = 1.0 if cuisine name or dish name appears in title, 0.0 otherwise. Ingredient/instruction signals = (matched keywords) / (threshold count), capped at 1.0. Threshold = 2 for ingredients, 1 for instructions.

- **Course**: `confidence = 0.40 * title_signal + 0.25 * ingredient_signal + 0.20 * description_signal + 0.15 * contextual_signal`. Contextual signal includes time-based cues (e.g., "overnight" → breakfast) and cooking method cues (e.g., "frosting" → dessert).

- **Diet**: Binary exclusion model. Start at 1.0 confidence for each diet category. For each ingredient, check if it violates the diet (e.g., "chicken" violates vegan). If violated → confidence = 0.0. If ingredient is ambiguous → reduce confidence by 0.2 per ambiguous ingredient. If no ingredients available → confidence = 0.0 for all diet tags (fail-safe for SC-006).

**Alternatives considered**:
- Flat keyword counting (equal weights): Doesn't account for title being a stronger signal than instructions. Produces lower accuracy for cuisine detection.
- Bayesian probability: Elegant but requires prior probability data we don't have. Better suited for learning-based systems (out of scope).

## R3: Ingredient Synonym/Alias Map

**Decision**: `HashMap<&str, &str>` mapping common ingredient name variations to canonical forms, compiled as static data.

**Rationale**: Ingredients appear in wildly varying forms ("AP flour", "all-purpose flour", "plain flour"). The synonym map normalizes these to canonical names that can be checked against dietary restriction lists. Using `&str` with static lifetime avoids allocation overhead.

**Design**:
- Map is a `phf` compile-time map or a lazy-initialized `HashMap` (prefer `HashMap` to avoid new dependency per Constitution III).
- Canonical names map to dietary properties via a second lookup: `HashMap<&str, Vec<DietaryProperty>>` where `DietaryProperty` indicates what diets the ingredient violates (e.g., "wheat flour" → `[ContainsGluten, NotGrainFree]`).
- Matching strategy: lowercase the ingredient `name` field, strip common prefixes ("organic ", "fresh ", "dried "), then look up in synonym map. If not found, use the name as-is for substring matching against canonical forms.

**Alternatives considered**:
- `phf` crate for compile-time perfect hash map: Faster lookup but adds a dependency. Standard `HashMap` with lazy_static is sufficient given small map size (~200-300 entries).
- Fuzzy matching (edit distance): Risk of false positives (e.g., "cream" matching "ice cream") which violates SC-006. Deterministic exact matching after normalization is safer.
- Regex patterns: Harder to maintain and audit. A flat map is more readable (Constitution I).

**Update**: Use `std::sync::LazyLock` (stable since Rust 1.80, but we target 1.77+). Fall back to `once_cell::sync::Lazy` or simply construct in a function call since maps are small enough to build on each call within <1ms. Actually, since `LazyLock` is stable in 1.80 and we target 1.77+, we'll use a simple function that returns the map. At ~300 entries this builds in microseconds and keeps the code simple with zero extra dependencies.

## R4: Tag Vocabulary Design

**Decision**: Predefined static arrays per domain, curated for real-world recipe coverage.

**Rationale**: Fixed vocabularies ensure consistent output, are auditable, and align with FR-017. Size is domain-appropriate rather than forcing uniform count.

**Vocabulary sizes**:
- **Cuisine**: ~35 labels (covers major world cuisines + regional variants)
- **Course**: ~12 labels (natural category count for meal types)
- **Diet**: ~15 labels (covers common dietary restrictions and preferences)

**Note**: The spec clarification stated "~30-50 per domain" as a guideline for comprehensive coverage. Cuisine naturally has the most labels. Course and diet have fewer natural categories — padding to 30+ would require artificial subdivisions that reduce signal quality. The intent of "comprehensive coverage including regional/niche" is met by the cuisine vocabulary.

**Alternatives considered**:
- External config files (JSON/TOML): Adds I/O, parsing, error handling for what is compile-time constant data. Over-engineered. Vocabularies change only when code is updated.
- Database-backed vocabularies: Out of scope (no storage layer yet). Would be relevant if dynamic vocabulary expansion were in scope.

## R5: Integration with Extraction Pipeline

**Decision**: Two Tauri commands — `tag_recipe` (standalone) and `extract_and_tag` (composed). Core tagging is a library function.

**Rationale**: FR-018 requires auto-tagging after extraction. FR-019 requires a separate on-demand command. The cleanest approach:
1. `tag_recipe_from_extracted(recipe: &ExtractedRecipe, refine: bool) -> TagSet` — pure library function, no Tauri dependency.
2. `tag_recipe` Tauri command — wraps the library function, accepts serialized recipe + optional refine flag.
3. `extract_and_tag` Tauri command — calls `extract_recipe_internal` then `tag_recipe_from_extracted`, returns a combined result. This satisfies FR-018 (auto-tag after extraction).

The extraction module is NOT modified. The tagging module imports `ExtractedRecipe` from `recipe_extraction::models` and composes at the command level.

**Alternatives considered**:
- Modify `extract_recipe` return type: Breaking change to existing command contract. Couples extraction and tagging modules.
- Frontend-orchestrated composition (call extract, then tag): Violates FR-018 ("no additional user action required") and adds IPC round-trip latency.
- Event-based (extraction emits event, tagging listens): Over-engineered for synchronous single-recipe flow. No concurrency benefit.

## R6: Heuristic Refinement Mode

**Decision**: Enhanced weighted scoring using combined signals across all recipe fields, with boosted weights for co-occurring indicators.

**Rationale**: The default rule-based mode checks individual signals independently. Heuristic refinement considers signal co-occurrence (e.g., if both "soy sauce" AND "rice noodles" appear, boost Asian cuisine confidence more than either alone). This is still deterministic and rule-based, just with richer scoring.

**Approach**:
- Same keyword vocabulary, but with a second-pass scoring function.
- Co-occurrence bonuses: When multiple indicators for the same tag appear across different fields (title + ingredients), apply a 1.2x multiplier to the combined score.
- Cross-domain signals: Course tagging can be informed by cuisine (e.g., "dim sum" → appetizer/snack).
- Re-normalize all scores after refinement to maintain [0.0, 1.0] range.

**Alternatives considered**:
- Separate ML model for refinement: Out of scope, adds dependency burden.
- User-trained weights: Out of scope (no learning from corrections).
