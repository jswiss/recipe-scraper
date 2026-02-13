# Research: Robots.txt Compliance

## R1: Robots.txt Parsing Approach

**Decision**: Use the `robotstxt` crate (v0.3.0)

**Rationale**:
- Zero external dependencies — aligns perfectly with the Minimal Dependencies principle
- Direct port of Google's production C++ robots.txt parser (which influenced RFC 9309)
- 1,354 lines of pure Rust, no unsafe code — auditable and forkable if needed
- 100% pass rate on Google's original test suite
- Supports all required features: `Allow`, `Disallow`, wildcard `*`, end anchor `$`, case-insensitive user-agent matching, longest-match precedence

**Alternatives Considered**:

| Option | Deps | Lines | RFC 9309 | Status |
|--------|------|-------|----------|--------|
| Custom implementation | 0 | 500-900 | TBD | High maintenance risk, edge case bugs |
| `texting_robots` 0.2.2 | 8 (98 transitive) | 2,388 | Partial | Stale (2024), heavy dep tree |
| **`robotstxt` 0.3.0** | **0** | **1,354** | **High** | Stable (spec is frozen), forkable |

**Note on maintenance**: Last updated 2021, but the robots.txt spec (RFC 9309, finalized 2022) hasn't changed since. The stable spec means lack of updates is expected, not concerning. If Rust compatibility breaks, the crate is small enough to vendor.

## R2: Cache Storage Approach

**Decision**: Add a `robots_cache` table to the existing SQLite database via a new migration (002)

**Rationale**:
- Reuses the existing `Database` struct and rusqlite connection (no new dependencies)
- Follows the established migration pattern from 001_initial.sql
- SQLite WAL mode supports concurrent reads for cached lookups
- 24-hour TTL implemented via `fetched_at` timestamp column + query-time filtering

**Alternatives Considered**:
- In-memory HashMap: Loses data on restart, violates Local First principle for offline support
- Separate SQLite file: Unnecessary complexity, existing database handles it

## R3: Integration with `ingest_url`

**Decision**: Separate `robots_compliance` module with its own Tauri command, also called as a gate inside `ingest_url`

**Rationale**:
- Single Responsibility: compliance checking is a distinct concern from URL fetching
- Frontend flexibility: can show compliance status before user commits to fetching
- Testability: compliance module can be tested independently
- The `ingest_url` command will import and call the compliance checker, returning a compliance-specific error variant if disallowed

## R4: `robotstxt` Crate API

**Decision**: Use `DefaultMatcher::one_agent_allowed_by_robots()` for single-URL checks

**API surface needed**:
```rust
use robotstxt::DefaultMatcher;

let mut matcher = DefaultMatcher::default();
let allowed = matcher.one_agent_allowed_by_robots(
    robots_body,    // &str — raw robots.txt content
    "RecipeScraper", // &str — user-agent to match
    url_path,        // &str — full URL to check
);
// Returns bool
```

**Limitation**: The crate does not parse `Crawl-delay` (not part of the original Google spec). We will implement a lightweight `Crawl-delay` extractor (~30 lines) ourselves since it's a simple `key: value` line parse for the matched user-agent group.
