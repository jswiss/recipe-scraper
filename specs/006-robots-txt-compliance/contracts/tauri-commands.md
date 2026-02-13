# Tauri Command Contracts: Robots.txt Compliance

## `check_robots_compliance`

Standalone compliance check for a URL. Returns a decision object without fetching page content.

### Signature

```rust
#[tauri::command]
pub async fn check_robots_compliance(
    url: String,
    client: State<'_, HttpClient>,
    db: State<'_, Database>,
) -> Result<RobotsDecision, RobotsError>
```

### Input

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| url | String | Yes | The URL to check compliance for |

### Output: `RobotsDecision`

```json
{
  "url": "https://www.allrecipes.com/recipe/12345",
  "allowed": true,
  "reason": "allowed by robots.txt",
  "crawl_delay_secs": 10.0,
  "matched_agent": "RecipeScraper",
  "source": "fresh"
}
```

### Error: `RobotsError`

```json
{
  "error_type": "fetch",
  "message": "Failed to fetch robots.txt: connection timeout",
  "url": "https://example.com/recipe/1",
  "details": "timed out after 10s"
}
```

### Behavior

1. Extract domain from `url`
2. Check `robots_cache` for a cached entry within 24h TTL
3. If cache miss or expired, fetch `https://{domain}/robots.txt`
4. Parse robots.txt, extract rules and crawl delay for "RecipeScraper" user-agent
5. Cache the result in `robots_cache`
6. Return `RobotsDecision` with allowed/disallowed status

### Edge Cases

| Scenario | Result |
|----------|--------|
| robots.txt returns 404 | `allowed: true`, reason: "robots.txt not found — allowed per RFC 9309" |
| robots.txt returns 5xx or times out | `allowed: false`, reason: "robots.txt unreachable" |
| robots.txt is empty | `allowed: true`, reason: "empty robots.txt — no restrictions" |
| robots.txt exceeds 500 KB | `allowed: true`, reason: "robots.txt oversized — treated as empty per policy" |
| URL is malformed | `RobotsError::InvalidUrl` |
| Offline with valid cache | Uses cached decision, `source: "cached"` |
| Offline with expired cache | Uses stale cache with reason suffix "(stale cache)" |
| Offline with no cache | `RobotsError::Fetch` with message about no network and no cache |

---

## Updated `ingest_url` (gate integration)

The existing `ingest_url` command is updated to call the compliance checker before fetching.

### Updated Behavior

1. Validate URL (existing)
2. Normalize URL (existing)
3. **NEW**: Call `check_robots_compliance` internally
4. If `allowed: false`, return `FetchError::RobotsDisallowed` (new variant)
5. If `allowed: true`, proceed with fetch (existing)

### New Error Variant: `FetchError::RobotsDisallowed`

```json
{
  "error_type": "robots_disallowed",
  "message": "Scraping disallowed by robots.txt",
  "url": "https://example.com/private/recipe",
  "reason": "disallowed by User-agent: *"
}
```

---

## Frontend Invocation Examples

```typescript
// Standalone compliance check
const decision = await invoke<RobotsDecision>('check_robots_compliance', {
  url: 'https://www.allrecipes.com/recipe/12345'
});

if (decision.allowed) {
  // Show green indicator, proceed with fetch
  if (decision.crawl_delay_secs) {
    // Optionally display crawl delay info
  }
} else {
  // Show warning: site does not allow scraping
}

// ingest_url now automatically checks compliance
try {
  const result = await invoke<FetchSuccess>('ingest_url', { url: '...' });
} catch (error) {
  if (error.error_type === 'robots_disallowed') {
    // Handle compliance rejection
  }
}
```
