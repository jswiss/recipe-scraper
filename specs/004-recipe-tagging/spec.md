# Feature Specification: Recipe Tagging and Categorization

**Feature Branch**: `004-recipe-tagging`
**Created**: 2026-02-07
**Status**: Draft
**Input**: User description: "Assign tags to a recipe in cuisine, course (e.g., breakfast, dessert), and diet (e.g., vegan, gluten-free) domains. Associate a confidence score with each tag. Consistent types for tags and confidence values."

## Clarifications

### Session 2026-02-07

- Q: What tagging approach should be used (rule-based, ML, or hybrid)? → A: Hybrid — rule-based keyword/pattern matching by default for all domains; optional user-triggered heuristic scoring refinement (e.g., a "refine tags" action) for improved accuracy on cuisine and course tags
- Q: How large should the initial tag vocabulary be per domain? → A: Large (~30-50 per domain) — comprehensive coverage including regional and niche categories
- Q: How should tagging integrate with the extraction pipeline? → A: Both — auto-tags with rule-based defaults after extraction; user can also re-trigger tagging (including heuristic refinement) on demand
- Q: What ingredient matching strategy should be used for diet tagging? → A: Normalized matching with synonym/alias map (e.g., "AP flour" → "wheat flour" → contains gluten) — deterministic, auditable, and safe for dietary restrictions

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic Cuisine Tagging (Priority: P1)

As a user, when I extract a recipe, I want the system to automatically assign cuisine tags (e.g., Italian, Mexican, Japanese) based on the recipe's ingredients, title, and preparation methods, so that I can organize and filter my recipes by cuisine.

**Why this priority**: Cuisine is the most universally recognized way to categorize recipes and provides the highest organizational value. Users intuitively think of recipes by cuisine first.

**Independent Test**: Can be fully tested by providing an extracted recipe with recognizable cuisine indicators (e.g., pasta, soy sauce, tortillas) and verifying appropriate cuisine tags are assigned with confidence scores.

**Acceptance Scenarios**:

1. **Given** an extracted recipe with clear cuisine indicators (e.g., title contains "Pad Thai", ingredients include fish sauce and rice noodles), **When** tagging is performed, **Then** the recipe receives a "Thai" cuisine tag with a confidence score above 0.7
2. **Given** an extracted recipe with ambiguous cuisine indicators, **When** tagging is performed, **Then** the recipe receives multiple cuisine tags with varying confidence scores reflecting the ambiguity
3. **Given** an extracted recipe with no recognizable cuisine indicators, **When** tagging is performed, **Then** no cuisine tags are assigned rather than guessing incorrectly

---

### User Story 2 - Automatic Course Tagging (Priority: P2)

As a user, I want the system to automatically assign course tags (e.g., breakfast, lunch, dinner, appetizer, dessert, snack, side dish) to extracted recipes, so that I can plan meals by course type.

**Why this priority**: Course categorization is essential for meal planning, which is a primary use case for recipe organization. It builds on the same tagging infrastructure as cuisine tagging.

**Independent Test**: Can be tested by providing extracted recipes with clear course indicators (e.g., "Chocolate Cake" for dessert, "Scrambled Eggs" for breakfast) and verifying appropriate course tags are assigned.

**Acceptance Scenarios**:

1. **Given** an extracted recipe titled "Blueberry Pancakes" with ingredients including flour, eggs, and blueberries, **When** tagging is performed, **Then** the recipe receives a "breakfast" course tag with a confidence score above 0.7
2. **Given** an extracted recipe that could serve as multiple courses (e.g., a salad could be appetizer, side dish, or lunch), **When** tagging is performed, **Then** the recipe receives multiple course tags with confidence scores reflecting likelihood for each
3. **Given** an extracted recipe with a clearly stated course in the title or description (e.g., "Appetizer: Bruschetta"), **When** tagging is performed, **Then** the stated course receives the highest confidence score

---

### User Story 3 - Automatic Diet Tagging (Priority: P3)

As a user, I want the system to automatically assign dietary tags (e.g., vegan, vegetarian, gluten-free, dairy-free, keto, paleo) to extracted recipes based on their ingredients, so that I can quickly find recipes matching my dietary needs.

**Why this priority**: Dietary filtering is critical for users with restrictions or preferences, but requires more nuanced ingredient analysis than cuisine or course tagging.

**Independent Test**: Can be tested by providing extracted recipes with known dietary characteristics (e.g., a recipe with no animal products should be tagged "vegan") and verifying correct diet tags are assigned.

**Acceptance Scenarios**:

1. **Given** an extracted recipe with only plant-based ingredients, **When** tagging is performed, **Then** the recipe receives "vegan" and "vegetarian" diet tags with high confidence scores
2. **Given** an extracted recipe containing wheat flour, **When** tagging is performed, **Then** the recipe does NOT receive a "gluten-free" diet tag
3. **Given** an extracted recipe with ambiguous ingredients (e.g., "butter" could be dairy or plant-based), **When** tagging is performed, **Then** the confidence score for relevant diet tags reflects the uncertainty
4. **Given** an extracted recipe with no ingredients list available, **When** tagging is performed, **Then** diet tags are either omitted or assigned with low confidence scores

---

### User Story 4 - View Tags with Confidence (Priority: P4)

As a user, I want to see the assigned tags alongside their confidence scores, so that I can understand how certain the system is about each categorization and decide whether to trust or override them.

**Why this priority**: Transparency about tagging confidence helps users trust the system and understand when manual correction may be needed. This is secondary to the actual tagging logic.

**Independent Test**: Can be tested by verifying the tagging output includes each tag paired with its confidence score in a consistent, readable format.

**Acceptance Scenarios**:

1. **Given** a recipe that has been tagged, **When** I view the tagging results, **Then** I see each tag grouped by domain (cuisine, course, diet) with its confidence score
2. **Given** a recipe with tags of varying confidence, **When** I view the results, **Then** tags within each domain are ordered by confidence score (highest first)

---

### Edge Cases

- What happens when a recipe has no title, no ingredients, and no steps? (Return empty tag sets for all domains with no tags rather than guessing)
- How does the system handle recipes with ingredients in different languages? (Tag based on available information; confidence scores reflect reduced certainty)
- What happens when an ingredient is ambiguous (e.g., "cream" could be dairy or coconut cream)? (Reflect ambiguity in confidence scores for diet tags)
- How does the system handle fusion recipes that span multiple cuisines? (Assign multiple cuisine tags with distributed confidence scores)
- What happens when confidence scores are very low for all potential tags in a domain? (Omit tags below a minimum confidence threshold rather than showing unreliable results)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST assign tags to a recipe across three domains: cuisine, course, and diet
- **FR-002**: System MUST associate a confidence score between 0.0 and 1.0 (inclusive) with every assigned tag
- **FR-003**: System MUST use consistent data types for all tags (string label) and confidence values (decimal number) across all three domains
- **FR-004**: System MUST accept an extracted recipe (from the recipe extraction module) as input for tagging
- **FR-005**: System MUST support assigning zero or more tags per domain for a single recipe
- **FR-006**: System MUST only assign tags that meet a minimum confidence threshold of 0.3
- **FR-007**: System MUST order tags within each domain by confidence score, highest first
- **FR-008**: System MUST derive cuisine tags from recipe title, ingredients, and preparation methods
- **FR-009**: System MUST derive course tags from recipe title, description, ingredients, and contextual cues
- **FR-010**: System MUST derive diet tags primarily from the ingredients list using normalized matching with a synonym/alias map (e.g., "AP flour" → "wheat flour" → contains gluten) to handle ingredient name variations deterministically
- **FR-011**: System MUST assign multiple tags per domain when a recipe matches more than one category (e.g., a recipe can be both "Italian" and "Mediterranean")
- **FR-012**: System MUST return a structured result containing all three tag domains, even if a domain has zero tags
- **FR-013**: System MUST complete tagging without requiring network access (local-first)
- **FR-014**: System MUST handle recipes with missing fields gracefully (e.g., no ingredients) by reducing confidence or omitting tags in affected domains
- **FR-015**: System MUST use rule-based keyword/pattern matching as the default tagging strategy for all three domains
- **FR-016**: System SHOULD provide an optional heuristic scoring refinement mode that users can trigger (e.g., "refine tags") to improve accuracy on cuisine and course tags using weighted multi-signal analysis
- **FR-017**: Each tag domain MUST define a comprehensive vocabulary of predefined labels stored as structured data (e.g., config or const definitions): ~35 for cuisine (including regional variants), ~12 for course (natural meal-type categories), ~15 for diet (common restrictions and preferences)
- **FR-018**: System MUST automatically run rule-based tagging immediately after recipe extraction completes, with no additional user action required
- **FR-019**: System MUST expose a separate on-demand tagging command that users can trigger to re-tag a recipe, including the optional heuristic refinement mode

### Key Entities

- **Tag**: A categorization label within a specific domain, paired with a confidence score (e.g., domain: "cuisine", label: "Italian", confidence: 0.85)
- **TagDomain**: One of three fixed categorization domains — cuisine, course, or diet
- **TagSet**: The complete collection of tags for a recipe, organized by domain, representing the full tagging result
- **Recipe** (input): An extracted recipe from the recipe extraction module, containing title, description, ingredients, steps, and other fields

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Tagging completes in under 1 second for a typical recipe
- **SC-002**: 90% of recipes with clear cuisine indicators receive the correct primary cuisine tag (highest confidence) as verified by manual review
- **SC-003**: 85% of recipes receive at least one correct course tag as verified by manual review
- **SC-004**: Diet tags correctly identify vegan, vegetarian, and gluten-free recipes with 90% accuracy based on ingredient analysis
- **SC-005**: All tags across all domains use the same consistent type structure (label + confidence score)
- **SC-006**: Zero false-positive diet tags for safety-critical dietary restrictions (e.g., a recipe with gluten is never tagged "gluten-free" with confidence above 0.3)

## Assumptions

- The input recipe is an already-extracted recipe from the recipe extraction module (feature 003)
- Cuisine, course, and diet vocabularies use a predefined, comprehensive set of labels (~30-50 per domain) covering common and regional/niche categories (not open-ended free text)
- Confidence scores are decimal values between 0.0 and 1.0, where 1.0 represents absolute certainty
- The minimum confidence threshold of 0.3 is a reasonable default to filter out noise
- Tagging operates locally using the recipe's own data fields (title, ingredients, steps, description) without external lookups
- Default tagging uses rule-based keyword/pattern matching; an optional heuristic refinement mode provides improved accuracy when user-triggered
- Tagging runs automatically after extraction (rule-based) and is also callable on-demand (including heuristic refinement)
- A recipe may receive zero tags in any domain if the system cannot determine a categorization with sufficient confidence
- Diet tagging is best-effort based on ingredient names matched through a normalized synonym/alias map; it cannot account for cross-contamination or manufacturing processes

## Out of Scope

- User-defined custom tags or categories beyond the three specified domains
- Manual tag editing or override by users (separate feature)
- Learning or improving tag accuracy from user corrections
- Tag-based recipe search or filtering (separate feature)
- Persisting tags to storage (separate feature)
- Expanding tag vocabularies dynamically
- Allergen detection beyond dietary category tagging
- Nutritional analysis or macro calculation from tags
