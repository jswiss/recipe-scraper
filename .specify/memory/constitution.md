<!--
SYNC IMPACT REPORT
==================
Version change: (new) → 1.0.0
Modified principles: N/A (initial creation)
Added sections:
  - Core Principles (6 principles)
  - Technology Preferences
  - Development Workflow
  - Governance
Removed sections: N/A (initial creation)
Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ No updates needed (Constitution Check section is dynamic)
  - .specify/templates/spec-template.md: ✅ No updates needed (requirements support constitution alignment)
  - .specify/templates/tasks-template.md: ✅ No updates needed (task structure is flexible)
Follow-up TODOs: None
-->

# Recipe Scraper Constitution

## Core Principles

### I. Readable & Simple Code

All code MUST prioritize human readability and simplicity. Code is written once but read many
times; optimize for the reader, not the writer.

- Variable and function names MUST be descriptive and self-documenting
- Functions MUST do one thing and do it well
- Complex logic MUST include comments explaining the "why", not the "what"
- Nesting depth SHOULD NOT exceed 3 levels; extract functions to flatten
- Prefer explicit over implicit behavior

**Rationale**: Code that is easy to read is easier to maintain, debug, and extend. Future
contributors (including future you) will thank you.

### II. AHA Programming (Avoid Hasty Abstractions)

Prefer duplication over premature abstraction. Only abstract when you have seen the pattern
repeat at least three times AND the abstraction provides clear value.

- MUST NOT create abstractions for hypothetical future use cases
- MUST tolerate some code duplication rather than creating wrong abstractions
- When abstracting, the abstraction MUST be simpler to use than the duplicated code
- Refactoring to abstraction is acceptable when the pattern is proven stable
- Delete unused abstractions aggressively

**Rationale**: Wrong abstractions are more costly than duplication. It's easier to abstract
duplicated code later than to untangle a premature abstraction.

### III. Minimal Dependencies

Limit external dependencies, but do not reinvent well-established functionality.

- MUST evaluate the maintenance burden, bundle size, and security surface of any new dependency
- MUST NOT add dependencies for trivial functionality that can be implemented in <50 lines
- SHOULD prefer dependencies with: active maintenance, small footprint, minimal transitive deps
- Standard library functionality MUST be preferred over third-party alternatives
- MUST document the rationale for any non-obvious dependency choice

**Rationale**: Dependencies create supply chain risk, version conflicts, and maintenance burden.
Every dependency is a liability that must be justified by its value.

### IV. Accessibility First

All user interfaces MUST achieve a high Lighthouse accessibility score. Accessibility is not
optional or a "nice to have" - it is a core requirement.

- All interactive elements MUST be keyboard navigable
- All images MUST have meaningful alt text
- Color MUST NOT be the only means of conveying information
- WCAG 2.1 Level AA compliance is the minimum target
- Semantic HTML MUST be used (proper heading hierarchy, landmarks, form labels)
- ARIA attributes MUST be used correctly when semantic HTML is insufficient
- Accessibility MUST be tested as part of the development process, not as an afterthought

**Rationale**: Accessible software serves all users, including those with disabilities. It also
improves usability for everyone (keyboard users, screen reader users, low-vision users, etc.).

### V. Monorepo Architecture with Open Source Preference

Follow monorepo architecture principles. Prefer open source solutions over cloud vendor
lock-in (AWS, Azure, GCP) where viable alternatives exist.

- Related packages and applications MUST live in a single repository
- Shared code MUST be extracted into internal packages rather than duplicated
- SHOULD prefer self-hostable open source tools over proprietary cloud services
- When cloud services are necessary, MUST prefer those with open standards and exit paths
- Build and deployment tooling MUST support the monorepo structure
- MUST NOT create separate repositories for tightly coupled components

**Rationale**: Monorepos simplify dependency management, atomic changes, and code sharing. Open
source preference reduces vendor lock-in, cost, and ensures long-term availability.

### VI. Local First Software

Follow Local First Software principles as defined by Ink & Switch. The user's device is the
primary source of truth; servers are secondary.

- Data MUST be stored locally on the user's device as the primary copy
- Application MUST work fully offline without degraded functionality
- Synchronization MUST happen seamlessly when connectivity is available
- Data MUST remain accessible regardless of service provider status
- User MUST retain full control and ownership of their data
- Collaboration features SHOULD use CRDTs or similar conflict-free approaches
- Privacy SHOULD be protected through end-to-end encryption where applicable
- Operations MUST respond immediately from local state (no loading spinners for local ops)

**Rationale**: Local-first design gives users true ownership of their data, ensures the
application works without network dependency, and provides the performance of native apps
while enabling collaboration when desired.

## Technology Preferences

Technology choices MUST align with constitutional principles:

- **Data Storage**: Local storage (IndexedDB, SQLite, filesystem) as primary; remote sync
  as secondary. No cloud-only storage for user data.
- **Dependencies**: Evaluate against Principle III criteria before adoption
- **UI Frameworks**: MUST support accessibility features out of the box
- **Sync Protocols**: Prefer open protocols (WebRTC, CRDTs) over proprietary sync services
- **Hosting**: Self-hostable solutions preferred; serverless/edge when cloud needed

## Development Workflow

All development MUST follow these workflow requirements:

- **Code Review**: All changes MUST be reviewed for constitution compliance
- **Accessibility Testing**: UI changes MUST include accessibility verification
- **Offline Testing**: Features MUST be tested in offline mode
- **Dependency Audit**: New dependencies MUST be documented with rationale
- **Simplicity Check**: Code MUST be reviewed for unnecessary complexity

## Governance

This constitution is the authoritative guide for all technical decisions in this project.
When in doubt, refer to these principles.

### Amendment Process

1. Proposed amendments MUST be documented with rationale
2. Amendments MUST be reviewed for impact on existing code and documentation
3. Version number MUST be updated according to semantic versioning:
   - **MAJOR**: Principle removal or incompatible redefinition
   - **MINOR**: New principle added or existing principle materially expanded
   - **PATCH**: Clarifications, wording improvements, typo fixes
4. Dependent templates MUST be reviewed and updated as needed

### Compliance

- All pull requests MUST be evaluated against constitution principles
- Constitution violations MUST be documented and justified in the Complexity Tracking section
- Regular audits SHOULD verify ongoing compliance

**Version**: 1.0.0 | **Ratified**: 2026-02-04 | **Last Amended**: 2026-02-04
