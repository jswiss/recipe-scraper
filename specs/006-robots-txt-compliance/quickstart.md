# Quickstart: Robots.txt Compliance

## Prerequisites

- Rust 1.77+ (stable toolchain)
- Existing recipe-scraper project built and running

## New Dependency

```toml
# Add to src-tauri/Cargo.toml [dependencies]
robotstxt = "0.3.0"
```

**Rationale**: Zero-dependency port of Google's production robots.txt parser. RFC 9309 compliant. See `research.md` for evaluation of alternatives.

## Module Structure

```
src-tauri/src/
├── robots_compliance/       # NEW module
│   ├── mod.rs               # Module exports
│   ├── models.rs            # RobotsDecision, RobotsError, CacheSource
│   ├── checker.rs           # Core logic: fetch, parse, cache, decide
│   ├── crawl_delay.rs       # Lightweight Crawl-delay parser (~30 lines)
│   └── commands.rs          # Tauri command: check_robots_compliance
├── lib.rs                   # Updated: register new command + gate in ingest_url
└── url_ingestion/
    └── models.rs            # Updated: add RobotsDisallowed error variant
```

## Database Migration

New file: `src-tauri/src/storage/migrations/002_robots_cache.sql`

```sql
CREATE TABLE IF NOT EXISTS robots_cache (
    domain TEXT PRIMARY KEY,
    raw_content TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('ok', 'not_found', 'unreachable', 'oversized'))
);
```

## Build & Test

```bash
cd src-tauri
cargo build        # Verify compilation with new dependency
cargo test         # Run all tests including new compliance tests
cargo clippy       # Lint check
```

## Key Integration Points

1. **Standalone command**: `check_robots_compliance` — call before or independent of fetching
2. **Gate in `ingest_url`**: Compliance check runs automatically; returns `RobotsDisallowed` error if blocked
3. **Cache layer**: SQLite-backed with 24h TTL, enables offline compliance checks
