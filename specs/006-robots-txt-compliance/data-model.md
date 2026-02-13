# Data Model: Robots.txt Compliance

## Entities

### RobotsDecision

The outcome of a compliance check for a single URL.

| Field | Type | Description |
|-------|------|-------------|
| url | String | The URL that was checked |
| allowed | bool | Whether scraping is permitted |
| reason | String | Human-readable explanation (e.g., "allowed by robots.txt", "disallowed by User-agent: *", "robots.txt not found — allowed per RFC 9309") |
| crawl_delay_secs | Option<f64> | Crawl delay in seconds, if specified for matched user-agent group |
| matched_agent | String | The user-agent group that matched (e.g., "RecipeScraper", "*") |
| source | CacheSource | Whether this decision came from a fresh fetch or cached data |

### CacheSource

Enum indicating where the robots.txt data came from.

| Variant | Description |
|---------|-------------|
| Fresh | Fetched from the site in this request |
| Cached | Loaded from SQLite cache (within TTL) |

### RobotsError

Error types for compliance checking.

| Variant | Fields | Description |
|---------|--------|-------------|
| Fetch | message, url, details | Failed to retrieve robots.txt (network/timeout) |
| Parse | message, url | robots.txt content could not be parsed |
| InvalidUrl | message, url | URL is malformed, cannot derive domain |

## SQLite Schema (Migration 002)

### `robots_cache` table

Stores parsed robots.txt content per domain with a TTL.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| domain | TEXT | PRIMARY KEY | Domain (host) the robots.txt belongs to |
| raw_content | TEXT | NOT NULL | Raw robots.txt body (for re-parsing if needed) |
| fetched_at | TEXT | NOT NULL | ISO 8601 timestamp of when robots.txt was fetched |
| status | TEXT | NOT NULL | Fetch result: "ok", "not_found", "unreachable", "oversized" |

### Cache TTL Logic

- **On check**: Query `robots_cache` for domain. If `fetched_at` is within 24 hours, use cached `raw_content`. Otherwise, re-fetch.
- **On fetch failure when cached exists**: If cache exists but is expired and re-fetch fails, use stale cache with a warning (graceful degradation).
- **Cleanup**: Entries older than 7 days are pruned on app startup.

## Relationships

```
RobotsDecision ← computed from → robots_cache row + URL path
robots_cache   ← keyed by     → domain (extracted from URL)
ingest_url     ← gated by     → RobotsDecision.allowed
```

## State Transitions

```
URL submitted
  → Extract domain
  → Check robots_cache for domain
    → Cache hit (within TTL): parse cached content, return decision
    → Cache miss / expired:
      → Fetch robots.txt from domain
        → 200 OK: parse, cache, return decision
        → 404/410: cache as "not_found", return allowed
        → 5xx/timeout/error: cache as "unreachable", return disallowed
        → Oversized (>500KB): cache as "oversized", return allowed
```
