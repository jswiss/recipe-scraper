# Feature Specification: Robots.txt Compliance

**Feature Branch**: `006-robots-txt-compliance`
**Created**: 2026-02-11
**Status**: Draft
**Input**: User description: "Respect robots.txt crawl policies. Evaluate if scraping is allowed before attempting. A decision object for each URL indicating allowed/disallowed and crawl delay."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Check If Scraping Is Allowed (Priority: P1)

Before the application fetches a recipe URL, it checks the site's robots.txt to determine whether scraping is permitted for that URL path. The compliance check is available as a standalone command (so the frontend can show compliance status proactively) and is also enforced as a gate within the existing `ingest_url` flow. If scraping is disallowed, the user receives a clear message explaining that the site does not permit automated access to that page.

**Why this priority**: This is the core value of the feature — without this check, the application cannot respect site policies, which is both an ethical and legal concern.

**Independent Test**: Can be fully tested by submitting a URL whose robots.txt disallows the path and verifying the system returns a "disallowed" decision without fetching the page content.

**Acceptance Scenarios**:

1. **Given** a URL whose robots.txt allows the path for our user-agent, **When** the compliance check runs, **Then** the decision object indicates "allowed" and includes any crawl delay specified.
2. **Given** a URL whose robots.txt disallows the path for our user-agent, **When** the compliance check runs, **Then** the decision object indicates "disallowed" and the application does not attempt to fetch the page content.
3. **Given** a URL whose robots.txt contains a wildcard disallow for all user-agents, **When** the compliance check runs, **Then** the decision object indicates "disallowed".

---

### User Story 2 - Respect Crawl Delay (Priority: P2)

When a site's robots.txt specifies a crawl delay, the application includes this information in the decision object so that callers can throttle requests appropriately. The crawl delay value is surfaced but enforcement of timing is left to the caller (e.g., the fetcher module).

**Why this priority**: Crawl delay compliance prevents aggressive request patterns that could get the application blocked or cause harm to target servers. It builds on the allowed/disallowed check.

**Independent Test**: Can be tested by checking a robots.txt with a `Crawl-delay` directive and verifying the decision object contains the correct delay value.

**Acceptance Scenarios**:

1. **Given** a robots.txt with `Crawl-delay: 10` for our user-agent, **When** the compliance check runs, **Then** the decision object includes a crawl delay of 10 seconds.
2. **Given** a robots.txt with no crawl delay directive, **When** the compliance check runs, **Then** the decision object indicates no crawl delay (defaults to none/zero).

---

### User Story 3 - Handle Missing or Unreachable robots.txt (Priority: P3)

When a site has no robots.txt file (HTTP 404) or the file cannot be fetched (network error, timeout), the application makes a sensible default decision. Per the robots.txt standard (RFC 9309), a missing robots.txt means all paths are allowed. An unreachable robots.txt (server error, timeout) means the application should treat the site as temporarily restricted.

**Why this priority**: Edge case handling ensures the application behaves correctly and safely even when robots.txt is unavailable.

**Independent Test**: Can be tested by pointing the compliance checker at a URL that returns 404 for robots.txt and verifying it returns "allowed", and at a URL that times out and verifying it returns "disallowed" with an appropriate reason.

**Acceptance Scenarios**:

1. **Given** a site that returns HTTP 404 for robots.txt, **When** the compliance check runs, **Then** the decision object indicates "allowed" (per RFC 9309 — no restrictions).
2. **Given** a site whose robots.txt cannot be fetched (timeout or server error), **When** the compliance check runs, **Then** the decision object indicates "disallowed" with a reason of "robots.txt unreachable".
3. **Given** a site that returns an empty robots.txt, **When** the compliance check runs, **Then** the decision object indicates "allowed" (empty file means no restrictions).

---

### Edge Cases

- What happens when robots.txt is malformed or contains invalid syntax? The system parses what it can and ignores invalid lines, following the "be liberal in what you accept" principle.
- What happens when robots.txt is extremely large (e.g., hundreds of KB)? The system enforces a maximum size limit (500 KB) and treats an oversized file as if it were empty (all paths allowed).
- What happens when a site uses a non-standard user-agent group that partially matches? The system follows RFC 9309 matching rules: case-insensitive prefix match on user-agent tokens.
- What happens when multiple user-agent groups apply? The system uses the most specific matching group per RFC 9309.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide the compliance check as a separate module with its own Tauri command, returning a decision object that callers can inspect independently.
- **FR-002**: System MUST return a decision object containing: allowed/disallowed status, crawl delay (if any), and the matched user-agent group.
- **FR-011**: System MUST integrate the compliance check as a gate within the existing `ingest_url` flow, automatically rejecting disallowed URLs before fetching.
- **FR-003**: System MUST match user-agent directives using case-insensitive prefix matching per RFC 9309.
- **FR-004**: System MUST treat a missing robots.txt (HTTP 404 or 410) as "all paths allowed" per RFC 9309.
- **FR-005**: System MUST treat an unreachable robots.txt (network error, HTTP 5xx, timeout) as "all paths disallowed".
- **FR-006**: System MUST support `Allow`, `Disallow`, and `Crawl-delay` directives.
- **FR-007**: System MUST use a configurable user-agent string for robots.txt matching (default: "RecipeScraper").
- **FR-008**: System MUST enforce a maximum robots.txt file size of 500 KB, treating oversized files as empty.
- **FR-009**: System MUST enforce a timeout when fetching robots.txt (default: 10 seconds) to avoid blocking on unresponsive servers.
- **FR-010**: System MUST cache parsed robots.txt per domain persistently (SQLite) with a 24-hour TTL, enabling offline compliance checks for previously-visited domains and avoiding redundant fetches.

### Key Entities

- **RobotsDecision**: The outcome of a compliance check for a single URL — contains the URL checked, whether access is allowed or disallowed, the reason for the decision, crawl delay (if any), and the matched user-agent group.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of recipe fetch attempts are preceded by a robots.txt compliance check — no URL is scraped without a decision.
- **SC-002**: The compliance check adds less than 2 seconds of latency to the first request for a given domain (subsequent requests for the same domain use cached results).
- **SC-003**: The system correctly identifies allowed/disallowed status for 95%+ of real-world recipe sites when compared against a reference robots.txt parser.
- **SC-004**: Every decision object contains all required fields (allowed status, crawl delay, reason) with no missing or null values.

## Clarifications

### Session 2026-02-11

- Q: Should cached robots.txt decisions persist across app restarts to support offline compliance checks? → A: Yes — persist to SQLite with a 24-hour TTL, enabling offline checks for previously-visited domains.
- Q: Should the compliance check be enforced inside `ingest_url` or be a separate module? → A: Separate module with its own Tauri command returning a decision object; `ingest_url` also calls it internally as a gate to block disallowed URLs.

## Assumptions

- The application uses a single user-agent string for all requests ("RecipeScraper" by default). This can be made configurable but does not need per-request user-agent switching.
- Caching of parsed robots.txt is persistent (SQLite) with a 24-hour TTL. Expired entries are re-fetched on next check. Offline users can rely on cached decisions for previously-visited domains until the TTL expires.
- The `Sitemap` directive in robots.txt is ignored — it is not relevant to compliance checking.
- Pattern matching in `Allow`/`Disallow` paths supports `*` (wildcard) and `$` (end-of-URL anchor) as specified in common robots.txt extensions and RFC 9309.
