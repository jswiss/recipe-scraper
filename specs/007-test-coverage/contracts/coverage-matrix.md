# Test Coverage Matrix

Maps each spec's acceptance scenarios to test status. "Existing" = already covered by current tests. "New" = needs to be added by this feature.

## Spec 001: URL Ingestion

| Scenario | Test Status | Test Location |
|----------|-------------|---------------|
| Valid HTTPS URL returns HTML | Existing | fetcher.rs (format only) |
| Valid URL with query params preserved | Existing | normalizer.rs::test_normalize_with_query_and_fragment |
| Trailing slash normalized | Existing | normalizer.rs::test_normalize_trailing_slash |
| Invalid URL returns error | Existing | validator.rs::test_missing_scheme |
| Missing protocol returns error | Existing | validator.rs::test_missing_scheme |
| Empty/whitespace returns error | Existing | validator.rs::test_empty_url, test_whitespace_url |
| Non-existent domain returns error | New | fetcher.rs (network — deferred per R-005) |
| HTTP 404 returns error | Existing | fetcher.rs::test_format_http_error_404 |
| Timeout returns error | New | fetcher.rs (network — deferred per R-005) |
| IDN URL converted to Punycode | Existing | normalizer.rs::test_normalize_idn_domain |
| Response >10MB rejected | New | commands.rs edge case test |
| Non-HTML content rejected | New | commands.rs edge case test |
| Redirects followed (up to 5) | New | fetcher.rs (network — deferred per R-005) |

## Spec 002: Rust/Tauri Refactor

Spec 002 acceptance scenarios overlap with 001 (same feature, Rust rewrite). All unique scenarios covered above.

## Spec 003: Recipe Extraction

| Scenario | Test Status | Test Location |
|----------|-------------|---------------|
| JSON-LD Recipe extracted with all fields | Existing | json_ld.rs::test_extract_from_jsonld |
| Microdata Recipe extracted with all fields | Existing | microdata.rs::test_extract_from_microdata |
| Multiple recipes → extract first | Existing | json_ld.rs::test_extract_with_graph |
| Missing field → null with justification | New | json_ld.rs edge case test |
| Missing prep time → null justified | New | json_ld.rs edge case test |
| Ambiguous servings → null justified | New | json_ld.rs edge case test |
| Nutrition extracted | New | json_ld.rs nutrition test |
| No nutrition → null justified | New | json_ld.rs edge case test |
| Images extracted | New | json_ld.rs image test |
| No images → null justified | New | json_ld.rs edge case test |
| Multiple images captured | New | json_ld.rs multi-image test |
| Malformed JSON-LD → graceful error | New | json_ld.rs malformed test |
| No recipe content → error | Existing | json_ld.rs::test_no_recipe_found |
| HowToStep instructions parsed | Existing | json_ld.rs::test_howto_instructions |

## Spec 004: Recipe Tagging

| Scenario | Test Status | Test Location |
|----------|-------------|---------------|
| Clear cuisine indicators → tag >0.7 | Existing | cuisine_tagger.rs::test_thai_recipe_clear_indicators |
| Ambiguous cuisine → multiple tags | Existing | cuisine_tagger.rs::test_italian_recipe_multiple_tags |
| No cuisine indicators → no tags | Existing | cuisine_tagger.rs::test_no_recognizable_indicators |
| Breakfast recipe tagged | Existing | course_tagger.rs::test_blueberry_pancakes_breakfast |
| Multi-course recipe → multiple tags | Existing | course_tagger.rs::test_salad_multiple_courses |
| Stated course in title → highest confidence | Existing | course_tagger.rs::test_explicit_course_in_title |
| Plant-based → vegan + vegetarian | Existing | diet_tagger.rs::test_all_plant_based_vegan_vegetarian |
| Wheat flour → not gluten-free | Existing | diet_tagger.rs::test_wheat_flour_not_gluten_free_sc006 |
| Ambiguous ingredients → reduced confidence | Existing | diet_tagger.rs::test_butter_ambiguous_reduces_dairy_free_confidence |
| No ingredients → diet tags omitted | Existing | diet_tagger.rs::test_no_ingredients_fail_safe |
| Tags grouped by domain | Existing | commands.rs::test_tag_recipe_valid_returns_all_domains |
| Tags ordered by confidence | Existing | commands.rs::test_tags_sorted_by_confidence_descending |
| Empty recipe → empty tag sets | Existing | commands.rs::test_completely_empty_recipe |
| Fusion recipe → multiple cuisines | Existing | commands.rs::test_fusion_recipe_multiple_cuisines |
| Tags below 0.5 threshold excluded | Existing | commands.rs::test_no_tag_below_threshold |

## Spec 005: Persistent State

| Scenario | Test Status | Test Location |
|----------|-------------|---------------|
| Save and retrieve round trip | Existing | repository.rs::save_and_get_round_trip |
| Duplicate URL updates existing | Existing | repository.rs::save_same_url_updates_instead_of_duplicating |
| Offline search by title | Existing | repository.rs::search_by_title_substring |
| Offline search by ingredient | Existing | repository.rs::search_by_ingredient_name |
| Filter by tag | Existing | repository.rs::search_filter_by_cuisine_tag |
| Combined search + filter | Existing | repository.rs::search_combined_text_and_tag |
| Soft delete | Existing | repository.rs::delete_recipe_sets_soft_delete |
| Partial update | Existing | repository.rs::update_recipe_changes_only_specified_fields |
| Export/import round trip | Existing | export.rs::export_import_round_trip |
| Malformed import entries skipped | Existing | export.rs::import_malformed_entries_skipped |
| Sync change log records mutations | Existing | sync.rs::change_log_records_mutations |
| JSONL export/import round trip | Existing | sync.rs::jsonl_export_import_round_trip |
| LWW conflict resolution | Existing | sync.rs::lww_merge_picks_newer_timestamp |
| Delete vs modify → modify wins | Existing | sync.rs::delete_vs_modify_restores_recipe |
| Backup round trip | **New** | backup.rs tests |
| Corrupted backup → error | **New** | backup.rs tests |
| Change log append on CRUD | **New** | change_log.rs tests |
| Change log query pending | **New** | change_log.rs tests |
| Change log mark synced | **New** | change_log.rs tests |
| Performance: 5000 recipes | Existing | repository.rs::performance_5000_recipes |
| Atomic transaction safety | Existing | repository.rs::atomic_transaction_safety |

## Spec 006: Robots.txt Compliance

| Scenario | Test Status | Test Location |
|----------|-------------|---------------|
| Allowed path → decision allowed | Existing | checker.rs (implicit via unit helpers) |
| Disallowed path → decision disallowed | New | checker.rs integration test |
| Wildcard disallow | New | checker.rs integration test |
| Crawl-delay parsed | Existing | crawl_delay.rs::test_specific_agent_delay |
| No crawl delay → none | Existing | crawl_delay.rs::test_no_delay |
| 404 robots.txt → allowed (RFC 9309) | Existing | checker.rs (status "missing" branch) |
| Unreachable robots.txt → disallowed | Existing | checker.rs (status "unreachable" branch) |
| Empty robots.txt → allowed | New | checker.rs test |
| Malformed robots.txt → parse what possible | New | crawl_delay.rs edge case test |
| Oversized robots.txt → treat as empty | Existing | checker.rs (status "oversized" branch) |
| Case-insensitive user-agent | Existing | crawl_delay.rs::test_case_insensitive_agent |
| ingest_url gates on robots | New | url_ingestion/commands.rs integration test |

## Summary

| Spec | Existing | New | Total |
|------|----------|-----|-------|
| 001 URL Ingestion | 8 | 3 (+3 deferred network) | 11 |
| 002 Rust Refactor | (covered by 001) | — | — |
| 003 Recipe Extraction | 5 | 8 | 13 |
| 004 Recipe Tagging | 15 | 0 | 15 |
| 005 Persistent State | 14 | 5 | 19 |
| 006 Robots Compliance | 6 | 4 | 10 |
| **Total** | **48** | **20** | **68** |

Plus integration pipeline tests (~5) and command wrapper tests (~10) = ~95 new tests total.
