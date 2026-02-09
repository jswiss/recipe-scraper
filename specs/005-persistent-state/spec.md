# Feature Specification: Persistent State

**Feature Branch**: `005-persistent-state`
**Created**: 2026-02-09
**Status**: Draft
**Input**: User description: "Spec 5 - Persistent state. It must have persistent state. This must work offline. It must allow for sync to other apps when online."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Save and Retrieve Recipes (Priority: P1)

A user scrapes a recipe from a website. After closing and reopening the app, the recipe (including all extracted fields, ingredients, instructions, nutrition, and tags) is still available. The user never loses a recipe they've previously scraped.

**Why this priority**: Without persistence, the app has zero long-term value. Every recipe is lost on restart. This is the foundational capability that all other stories depend on.

**Independent Test**: Can be fully tested by scraping a recipe, closing the app, reopening it, and verifying the recipe appears with all its data intact. Delivers the core value of a recipe collection.

**Acceptance Scenarios**:

1. **Given** a user has scraped a recipe, **When** they close and reopen the app, **Then** the recipe appears in their collection with all fields preserved (title, description, ingredients, instructions, times, servings, images, nutrition, tags, source URL).
2. **Given** a user has scraped multiple recipes over several sessions, **When** they open the app, **Then** all previously scraped recipes are available.
3. **Given** a user scrapes a recipe from a URL they've previously scraped, **When** the scrape completes, **Then** the system updates the existing recipe rather than creating a duplicate.

---

### User Story 2 - Browse and Search Saved Recipes Offline (Priority: P2)

A user opens the app without an internet connection. They can browse their entire recipe collection, view full recipe details, and search/filter by tags, title, or ingredients. All local operations are instant with no loading indicators.

**Why this priority**: The local-first principle demands that core functionality works fully offline. Browsing saved recipes is the primary use case after initial scraping.

**Independent Test**: Can be tested by saving several recipes while online, disconnecting from the internet, and verifying full browse/search/filter functionality with no degradation.

**Acceptance Scenarios**:

1. **Given** the user has saved recipes and is offline, **When** they open the app, **Then** all saved recipes are available for browsing.
2. **Given** the user is offline, **When** they search or filter recipes by tag, title, or ingredient, **Then** results appear instantly (no loading spinners for local operations).
3. **Given** the user is offline, **When** they view a recipe's full details, **Then** all persisted data is displayed (including nutrition, instructions, and tags).

---

### User Story 3 - Export Recipes for Use in Other Apps (Priority: P3)

A user wants to share their recipe collection (or individual recipes) with another application or another person. They can export recipes in a standard, interoperable format whether online or offline. The exported data can be imported by other recipe applications that support the same format.

**Why this priority**: Interoperability prevents vendor lock-in and supports the open-source principle. Users must own their data and move it freely.

**Independent Test**: Can be tested by exporting a recipe, importing it into a different recipe application (or a text editor for format validation), and verifying data completeness.

**Acceptance Scenarios**:

1. **Given** a user selects one or more recipes, **When** they choose to export, **Then** the recipes are saved in a standard, human-readable format that preserves all recipe data.
2. **Given** a user exports recipes while offline, **When** the export completes, **Then** the export file is saved locally and available immediately.
3. **Given** a user has exported recipes from this app, **When** they import the file into another recipe application that supports the same format, **Then** the recipe data transfers correctly.

---

### User Story 4 - Sync Recipes Across Devices (Priority: P4)

A user has the app installed on multiple devices. When online, recipes saved on one device eventually appear on all other devices. Sync happens automatically in the background without blocking the user. If the same recipe is modified on two devices while offline, the system resolves the conflict without losing data.

**Why this priority**: Multi-device sync is valuable but secondary to local persistence and offline capability. It enhances the experience but is not required for core functionality.

**Independent Test**: Can be tested by saving a recipe on Device A, waiting for sync, and verifying it appears on Device B with all data intact.

**Acceptance Scenarios**:

1. **Given** a user saves a recipe on Device A while online, **When** they open the app on Device B (also online), **Then** the recipe appears on Device B.
2. **Given** a user modifies a recipe on Device A while offline and modifies the same recipe on Device B while offline, **When** both devices come online, **Then** the system merges changes without losing either modification.
3. **Given** sync is in progress, **When** the user interacts with the app, **Then** sync happens in the background and does not block any local operations.

---

### Edge Cases

- What happens when local storage is full or near capacity? The system notifies the user and prevents data loss by refusing new writes rather than silently failing.
- What happens if a recipe's source URL becomes unavailable after saving? The saved recipe data remains fully intact; persistence is independent of the source website.
- What happens during a sync conflict where the same recipe field was edited on two devices? The system automatically resolves using last-write-wins at the per-field level, preserving non-conflicting changes from both devices.
- What happens if the app crashes mid-save? The system uses atomic writes so data is either fully saved or not saved at all; no partial or corrupt states.
- What happens when a user deletes a recipe on one device while another device is offline? If the offline device did not modify the recipe, the deletion syncs normally. If the offline device modified the recipe, the modification wins and the recipe is restored with the edits on all devices (the user can re-delete if intended).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist all recipe data locally on the user's device, including title, description, ingredients, instructions, prep/cook times, servings, images, nutrition info, extraction source, source URL, and tags.
- **FR-002**: System MUST load all persisted recipes on app startup without requiring a network connection.
- **FR-003**: System MUST deduplicate recipes by source URL; re-scraping the same URL updates the existing recipe rather than creating a new entry.
- **FR-004**: System MUST perform all local data operations (read, write, search, filter) without network access.
- **FR-005**: System MUST NOT display loading spinners or progress indicators for local data operations.
- **FR-006**: System MUST support exporting individual recipes or the entire collection as JSON using the schema.org/Recipe vocabulary.
- **FR-007**: System MUST support importing recipes from schema.org/Recipe JSON (the same format used for export).
- **FR-008**: System MUST preserve data integrity during unexpected shutdowns (atomic writes; no partial saves).
- **FR-009**: System MUST allow users to delete individual recipes from their collection.
- **FR-015**: System MUST allow users to edit any field of a saved recipe (title, description, ingredients, instructions, times, servings, tags). Edits are persisted and participate in sync.
- **FR-016**: System MUST provide a personal notes field on each recipe for free-form user annotations. Notes are persisted and synced alongside all other recipe data.
- **FR-010**: System MUST track when each recipe was first saved and last updated.
- **FR-011**: System MUST allow users to search and filter recipes by title, ingredients, and tags (cuisine, course, diet) while offline.
- **FR-012**: System MUST sync recipe data across devices by storing its data file in a cloud-synced location (e.g., iCloud). The OS/cloud storage layer handles file transport; the app does not implement its own sync protocol.
- **FR-013**: System MUST handle sync conflicts automatically using a last-write-wins strategy at the per-field level. When the same recipe is modified on multiple devices, each field resolves independently to the most recent edit, preserving non-conflicting changes from both devices. Conflict resolution operates at the application layer on top of the file-based sync.
- **FR-014**: System MUST allow users to back up and restore their entire recipe collection.

### Key Entities

- **Recipe**: The central entity. Represents a single scraped recipe with all extracted fields (title, description, ingredients, instructions, times, servings, images, nutrition), its source URL, extraction metadata, assigned tags, a personal notes field, and persistence timestamps (created/updated). All fields are user-editable after saving. Recipes are always sourced from a scrape (no manual creation).
- **Tag**: A categorization label with a confidence score, belonging to a domain (cuisine, course, or diet). Associated with a recipe. Users may override auto-generated tags.
- **Ingredient**: A structured component of a recipe with name, optional quantity, optional unit, and raw text.
- **Instruction**: An ordered preparation step within a recipe, with a step number and text.
- **Sync State**: Metadata tracking per-recipe modification timestamps to support conflict resolution when the shared data file is updated by multiple devices via cloud storage.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can close and reopen the app and see 100% of previously saved recipes with all data intact.
- **SC-002**: All local operations (browsing, searching, filtering, viewing recipe details) complete in under 100 milliseconds with no visible loading indicators, even with a collection of 5,000 recipes.
- **SC-003**: The app functions fully offline for all local operations (browse, search, filter, view, export, import) with zero network dependency.
- **SC-004**: Exported recipes can be successfully imported into the app on a different device with 100% data fidelity.
- **SC-005**: When two devices are online, a recipe saved on one device appears on the other within 30 seconds.
- **SC-006**: No user data is lost during sync conflicts; all conflicting changes are preserved or merged.
- **SC-007**: Users can back up their entire collection and restore it on a fresh install with 100% data fidelity.

## Clarifications

### Session 2026-02-09

- Q: Can users manually create new recipes or edit scraped recipe fields after saving? → A: Users can edit scraped recipe fields after saving but cannot manually create new recipes from scratch. Each recipe also has a personal notes field for user annotations.
- Q: When one device deletes a recipe and another device edits it, which wins? → A: Modify wins. The recipe is restored with the edits on all devices; the user can re-delete if intended.
- Q: What is the expected upper bound on recipe collection size? → A: Up to 5,000 recipes. Performance targets (SC-002) must hold at this scale.
- Q: What specific export/import format should be used? → A: JSON using schema.org/Recipe vocabulary, the same structured data format the app already extracts.
- Q: How does sync work between devices? → A: File-based sync via iCloud. The app stores its SQLite data file in an iCloud-accessible directory; the OS handles file transport. No custom sync server or protocol needed.

## Assumptions

- The user's device has sufficient local storage for their recipe collection. A collection may contain up to 5,000 recipes. Recipes are primarily text data with image URLs (not image files stored locally), so storage requirements are modest.
- Export/import uses JSON with the schema.org/Recipe vocabulary — the same structured data format the app already extracts from websites.
- Sync is opt-in via the user's existing cloud storage (iCloud); the app does not require account creation or a custom sync server.
- Image URLs are persisted but images themselves are not cached locally (image caching may be a separate feature).
- The sync mechanism is file-based: the app stores its SQLite data file in an iCloud-accessible directory, and iCloud handles file transport across devices. No custom sync server or peer-to-peer protocol is needed.
