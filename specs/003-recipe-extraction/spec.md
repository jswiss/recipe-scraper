# Feature Specification: Recipe Structure Extraction

**Feature Branch**: `003-recipe-extraction`
**Created**: 2026-02-04
**Status**: Draft
**Input**: User description: "Produce a structured data object for a recipe with title, description, ingredients, steps, times (prep, cook), servings, images (optional), optional nutrition. Should prioritize embedded machine-readable metadata if present (e.g., JSON-LD). All fields are present with valid content or marked explicitly null with justification."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Extract Recipe from Structured Data (Priority: P1)

As a user, I want the system to extract recipe information from machine-readable metadata (JSON-LD, Microdata) when available, so that I get the most accurate and complete recipe data with minimal parsing errors.

**Why this priority**: Machine-readable metadata is the most reliable source of recipe data, as it's explicitly structured by the recipe publisher. This provides the highest accuracy and should be the primary extraction method.

**Independent Test**: Can be fully tested by providing HTML containing valid JSON-LD recipe schema and verifying all recipe fields are correctly extracted.

**Acceptance Scenarios**:

1. **Given** HTML containing valid JSON-LD with Recipe schema, **When** I request extraction, **Then** all available fields are populated from the structured data
2. **Given** HTML containing Microdata with Recipe schema, **When** I request extraction, **Then** all available fields are populated from the structured data
3. **Given** HTML with multiple recipe schemas, **When** I request extraction, **Then** the system extracts the primary/first recipe

---

### User Story 2 - Extract Recipe from HTML Content (Priority: P2)

As a user, when no machine-readable metadata is available, I want the system to extract recipe information from the HTML content itself, so that I can still get recipe data from older or less structured websites.

**Why this priority**: Many recipe websites don't use structured data, so HTML fallback parsing is essential for broad compatibility. This is secondary to structured data as it's less reliable.

**Independent Test**: Can be tested by providing HTML without any structured data markup and verifying reasonable extraction from visible content.

**Acceptance Scenarios**:

1. **Given** HTML without structured data but with recognizable recipe content, **When** I request extraction, **Then** the system attempts to extract fields from the page content
2. **Given** HTML where some fields cannot be determined, **When** I request extraction, **Then** those fields are marked as null with a justification explaining why

---

### User Story 3 - Handle Incomplete or Missing Data (Priority: P3)

As a user, I want clear feedback when recipe data is incomplete or missing, so that I understand what information was successfully extracted and what couldn't be found.

**Why this priority**: Transparency about extraction quality helps users decide whether the extracted data is useful and what might need manual correction.

**Independent Test**: Can be tested by providing HTML with partial recipe information and verifying null fields include appropriate justifications.

**Acceptance Scenarios**:

1. **Given** a recipe page missing prep time, **When** I request extraction, **Then** prep_time is null with justification "Not found in source"
2. **Given** a recipe page with ambiguous servings, **When** I request extraction, **Then** servings is null with justification explaining the ambiguity
3. **Given** a recipe page with nutrition information, **When** I request extraction, **Then** nutrition data is included in the result
4. **Given** a recipe page without nutrition information, **When** I request extraction, **Then** nutrition is null with justification "Not provided"

---

### User Story 4 - Extract Images (Priority: P4)

As a user, I want recipe images to be extracted when available, so that I can visually identify the dish.

**Why this priority**: Images enhance the user experience but are not essential for the core recipe data. They're optional and supplementary.

**Independent Test**: Can be tested by providing recipe HTML with images and verifying image URLs are extracted.

**Acceptance Scenarios**:

1. **Given** a recipe with associated images, **When** I request extraction, **Then** image URLs are included in the result
2. **Given** a recipe without images, **When** I request extraction, **Then** images is null with justification "No images found"
3. **Given** multiple images in the recipe, **When** I request extraction, **Then** all relevant image URLs are captured

---

### Edge Cases

- What happens when the page contains multiple recipes? (Extract the primary/first recipe)
- How does the system handle malformed JSON-LD? (Fall back to HTML parsing)
- What happens when ingredient quantities are missing? (Include ingredient name with null quantity)
- How are cooking times in different formats handled? (Normalize to minutes)
- What happens when the page is not a recipe at all? (Return error indicating no recipe found)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST check for JSON-LD Recipe schema before attempting HTML parsing
- **FR-002**: System MUST check for Microdata Recipe schema if JSON-LD is not present
- **FR-003**: System MUST extract title as a non-empty string or mark as null with justification
- **FR-004**: System MUST extract description as a string or mark as null with justification
- **FR-005**: System MUST extract ingredients as a list of structured items (name, quantity, unit) or raw strings
- **FR-006**: System MUST extract steps as an ordered list of instruction strings
- **FR-007**: System MUST extract prep_time and cook_time in minutes or mark as null with justification
- **FR-008**: System MUST extract servings as a number or range, or mark as null with justification
- **FR-009**: System MUST extract image URLs when available, or mark as null with justification
- **FR-010**: System MUST extract nutrition information when available, or mark as null with justification
- **FR-011**: System MUST return all fields in every response (no omitted fields)
- **FR-012**: System MUST provide a justification string for every null field
- **FR-013**: System MUST accept HTML content as input (from URL ingestion module)
- **FR-014**: System MUST return an error if no recipe content can be identified

### Key Entities

- **Recipe**: The complete extracted recipe containing all fields (title, description, ingredients, steps, times, servings, images, nutrition)
- **Ingredient**: A recipe ingredient with optional structured data (name, quantity, unit) or raw text
- **Instruction**: A single step in the recipe preparation process
- **NutritionInfo**: Optional nutritional data (calories, fat, protein, carbohydrates, etc.)
- **ExtractedField**: A field that is either a value or null with justification

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Recipe extraction completes in under 2 seconds for typical recipe pages
- **SC-002**: 95% of recipes with JSON-LD schema have all available fields correctly extracted
- **SC-003**: 80% of recipes without structured data have title, ingredients, and steps successfully extracted
- **SC-004**: All null fields include a human-readable justification
- **SC-005**: System correctly identifies and rejects non-recipe pages 95% of the time
- **SC-006**: Extracted recipes can be serialized to a consistent data format for downstream use

## Assumptions

- HTML content is provided by the URL ingestion module (already fetched)
- JSON-LD and Microdata parsing follows schema.org Recipe vocabulary
- Times are normalized to minutes for consistency
- Image URLs are returned as absolute URLs
- Nutrition values use standard units (calories, grams for macros)
- The system processes one recipe per extraction request (first/primary recipe if multiple exist)

## Out of Scope

- Recipe storage or persistence (separate feature)
- Recipe deduplication or merging
- User interface for displaying extracted recipes
- Recipe editing or manual correction
- Translation of recipes to different languages
- Conversion between measurement units (metric/imperial)
- Video extraction from recipe pages
