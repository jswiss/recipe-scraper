use reqwest::Client;
use std::time::Duration;

use crate::robots_compliance::crawl_delay::parse_crawl_delay;
use crate::robots_compliance::models::{CacheSource, RobotsDecision, RobotsError};
use crate::storage::change_log::now_utc;
use crate::storage::Database;

/// User-agent string used for robots.txt matching.
pub const USER_AGENT: &str = "RecipeScraper";

const ROBOTS_FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ROBOTS_SIZE: usize = 500 * 1024; // 500 KB
const CACHE_TTL_HOURS: i64 = 24;

/// Result of fetching a robots.txt file.
struct FetchResult {
    raw_content: String,
    status: String, // "ok", "not_found", "unreachable", "oversized"
}

/// Extracts the domain (host) from a URL string.
fn extract_domain(url: &str) -> Result<String, RobotsError> {
    let parsed = url::Url::parse(url).map_err(|e| RobotsError::InvalidUrl {
        message: format!("Invalid URL: {e}"),
        url: url.to_string(),
    })?;

    parsed
        .host_str()
        .map(|h| h.to_string())
        .ok_or_else(|| RobotsError::InvalidUrl {
            message: "URL has no host".to_string(),
            url: url.to_string(),
        })
}

/// Constructs the robots.txt URL for a domain.
fn robots_url(url: &str) -> Result<String, RobotsError> {
    let parsed = url::Url::parse(url).map_err(|e| RobotsError::InvalidUrl {
        message: format!("Invalid URL: {e}"),
        url: url.to_string(),
    })?;

    let scheme = parsed.scheme();
    let host = parsed.host_str().ok_or_else(|| RobotsError::InvalidUrl {
        message: "URL has no host".to_string(),
        url: url.to_string(),
    })?;

    let port = parsed
        .port()
        .map(|p| format!(":{p}"))
        .unwrap_or_default();

    Ok(format!("{scheme}://{host}{port}/robots.txt"))
}

/// Fetches robots.txt from the network. Does NOT use cache.
async fn fetch_robots_txt(client: &Client, url: &str) -> FetchResult {
    let robots = match robots_url(url) {
        Ok(u) => u,
        Err(_) => {
            return FetchResult {
                raw_content: String::new(),
                status: "unreachable".to_string(),
            }
        }
    };

    let request = client
        .get(&robots)
        .timeout(ROBOTS_FETCH_TIMEOUT)
        .send()
        .await;

    let response = match request {
        Ok(resp) => resp,
        Err(_) => {
            return FetchResult {
                raw_content: String::new(),
                status: "unreachable".to_string(),
            }
        }
    };

    let status_code = response.status().as_u16();

    // 404 or 410 — robots.txt not found
    if status_code == 404 || status_code == 410 {
        return FetchResult {
            raw_content: String::new(),
            status: "not_found".to_string(),
        };
    }

    // 5xx — server error
    if response.status().is_server_error() {
        return FetchResult {
            raw_content: String::new(),
            status: "unreachable".to_string(),
        };
    }

    // Non-success status
    if !response.status().is_success() {
        return FetchResult {
            raw_content: String::new(),
            status: "unreachable".to_string(),
        };
    }

    // Check content length before downloading
    if let Some(len) = response.content_length() {
        if len as usize > MAX_ROBOTS_SIZE {
            return FetchResult {
                raw_content: String::new(),
                status: "oversized".to_string(),
            };
        }
    }

    // Read body
    let body = match response.text().await {
        Ok(text) => text,
        Err(_) => {
            return FetchResult {
                raw_content: String::new(),
                status: "unreachable".to_string(),
            }
        }
    };

    // Check actual size after download
    if body.len() > MAX_ROBOTS_SIZE {
        return FetchResult {
            raw_content: String::new(),
            status: "oversized".to_string(),
        };
    }

    FetchResult {
        raw_content: body,
        status: "ok".to_string(),
    }
}

/// Cached entry from the robots_cache table.
struct CachedEntry {
    raw_content: String,
    status: String,
    fetched_at: String,
}

/// Returns true if the fetched_at timestamp is within the TTL window.
fn is_within_ttl(fetched_at: &str) -> bool {
    // Parse fetched_at as an ISO 8601 timestamp and compare to now.
    // Simple approach: compare strings since now_utc() returns ISO 8601
    // and TTL is 24 hours. Use a numeric comparison on the timestamp.
    let now = now_utc();

    // Parse both as simple strings — ISO 8601 sorts lexicographically
    // so we can use string comparison for a rough check.
    // For a precise check, we'd parse into a proper datetime, but since
    // both are in the same format (YYYY-MM-DDTHH:MM:SS.xxxZ), this works.
    // We need to check if (now - fetched_at) < 24 hours.

    // Parse year, month, day, hour from the timestamp strings
    if let (Some(fetched_ts), Some(now_ts)) = (parse_timestamp(fetched_at), parse_timestamp(&now))
    {
        let diff_secs = now_ts.saturating_sub(fetched_ts);
        diff_secs < (CACHE_TTL_HOURS as u64 * 3600)
    } else {
        false // Can't parse, treat as expired
    }
}

/// Parses an ISO 8601 timestamp string into seconds since epoch (approximate).
fn parse_timestamp(ts: &str) -> Option<u64> {
    // Expected format: "YYYY-MM-DDTHH:MM:SS.nnnZ" or "YYYY-MM-DD HH:MM:SS"
    // We only need approximate seconds for TTL comparison.
    let ts = ts.replace('T', " ").replace('Z', "");
    let parts: Vec<&str> = ts.split(' ').collect();
    if parts.len() < 2 {
        return None;
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() < 3 || time_parts.len() < 3 {
        return None;
    }

    let year: u64 = date_parts[0].parse().ok()?;
    let month: u64 = date_parts[1].parse().ok()?;
    let day: u64 = date_parts[2].parse().ok()?;
    let hour: u64 = time_parts[0].parse().ok()?;
    let min: u64 = time_parts[1].parse().ok()?;
    let sec_str = time_parts[2].split('.').next()?;
    let sec: u64 = sec_str.parse().ok()?;

    // Approximate seconds since a fixed epoch (good enough for TTL comparison)
    Some(year * 31_536_000 + month * 2_592_000 + day * 86_400 + hour * 3600 + min * 60 + sec)
}

/// Looks up the cache for a domain, returns None if no entry or expired.
fn get_cached(db: &Database, domain: &str) -> Option<CachedEntry> {
    let conn = db.conn.lock().ok()?;
    let mut stmt = conn
        .prepare("SELECT raw_content, status, fetched_at FROM robots_cache WHERE domain = ?1")
        .ok()?;

    stmt.query_row(rusqlite::params![domain], |row| {
        Ok(CachedEntry {
            raw_content: row.get(0)?,
            status: row.get(1)?,
            fetched_at: row.get(2)?,
        })
    })
    .ok()
}

/// Writes or updates a cache entry.
fn upsert_cache(db: &Database, domain: &str, content: &str, status: &str) {
    let now = now_utc();
    if let Ok(conn) = db.conn.lock() {
        let _ = conn.execute(
            "INSERT INTO robots_cache (domain, raw_content, fetched_at, status)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(domain) DO UPDATE SET
               raw_content = excluded.raw_content,
               fetched_at = excluded.fetched_at,
               status = excluded.status",
            rusqlite::params![domain, content, now, status],
        );
    }
}

/// Gets robots.txt content from cache or fetches from network.
/// Returns (raw_content, status, CacheSource).
async fn get_or_fetch_robots(
    client: &Client,
    db: &Database,
    url: &str,
) -> Result<(String, String, CacheSource), RobotsError> {
    let domain = extract_domain(url)?;

    // Check cache first
    if let Some(cached) = get_cached(db, &domain) {
        if is_within_ttl(&cached.fetched_at) {
            return Ok((cached.raw_content, cached.status, CacheSource::Cached));
        }

        // Cache is expired — try to re-fetch, fall back to stale cache on failure
        let result = fetch_robots_txt(client, url).await;
        if result.status == "unreachable" {
            // Stale cache fallback
            return Ok((cached.raw_content, cached.status, CacheSource::Cached));
        }

        upsert_cache(db, &domain, &result.raw_content, &result.status);
        return Ok((result.raw_content, result.status, CacheSource::Fresh));
    }

    // No cache entry — fetch from network
    let result = fetch_robots_txt(client, url).await;

    // If unreachable and no cache at all, report the error
    if result.status == "unreachable" {
        upsert_cache(db, &domain, &result.raw_content, &result.status);
        return Ok((result.raw_content, result.status, CacheSource::Fresh));
    }

    upsert_cache(db, &domain, &result.raw_content, &result.status);
    Ok((result.raw_content, result.status, CacheSource::Fresh))
}

/// Core compliance check: fetches/caches robots.txt, parses it, returns a decision.
pub async fn check_compliance(
    client: &Client,
    db: &Database,
    url: &str,
) -> Result<RobotsDecision, RobotsError> {
    let (raw_content, status, source) = get_or_fetch_robots(client, db, url).await?;

    // Extract crawl delay from raw content (if available)
    let crawl_delay_secs = if status == "ok" && !raw_content.is_empty() {
        parse_crawl_delay(&raw_content, USER_AGENT)
    } else {
        None
    };

    // Map status to decision
    let (allowed, reason, matched_agent) = match status.as_str() {
        "not_found" => (
            true,
            "robots.txt not found \u{2014} allowed per RFC 9309".to_string(),
            "*".to_string(),
        ),
        "unreachable" => (
            false,
            "robots.txt unreachable".to_string(),
            "*".to_string(),
        ),
        "oversized" => (
            true,
            "robots.txt oversized \u{2014} treated as empty per policy".to_string(),
            "*".to_string(),
        ),
        "ok" => {
            if raw_content.is_empty() {
                (
                    true,
                    "empty robots.txt \u{2014} no restrictions".to_string(),
                    "*".to_string(),
                )
            } else {
                // Use robotstxt crate to check
                let mut matcher = robotstxt::DefaultMatcher::default();
                let is_allowed =
                    matcher.one_agent_allowed_by_robots(&raw_content, USER_AGENT, url);

                // Determine which agent group matched
                // Try specific agent first, then wildcard
                let mut specific_matcher = robotstxt::DefaultMatcher::default();
                let specific_allowed =
                    specific_matcher.one_agent_allowed_by_robots(&raw_content, USER_AGENT, url);
                let mut wildcard_matcher = robotstxt::DefaultMatcher::default();
                let wildcard_allowed =
                    wildcard_matcher.one_agent_allowed_by_robots(&raw_content, "*", url);

                let matched = if specific_allowed != wildcard_allowed {
                    USER_AGENT.to_string()
                } else {
                    "*".to_string()
                };

                let reason = if is_allowed {
                    "allowed by robots.txt".to_string()
                } else {
                    format!("disallowed by User-agent: {matched}")
                };

                (is_allowed, reason, matched)
            }
        }
        _ => (
            false,
            format!("unknown robots.txt status: {status}"),
            "*".to_string(),
        ),
    };

    // Stale cache suffix
    let reason = if source == CacheSource::Cached
        && !is_within_ttl(
            &get_cached(
                db,
                &extract_domain(url).unwrap_or_default(),
            )
            .map(|c| c.fetched_at)
            .unwrap_or_default(),
        )
    {
        format!("{reason} (stale cache)")
    } else {
        reason
    };

    Ok(RobotsDecision {
        url: url.to_string(),
        allowed,
        reason,
        crawl_delay_secs,
        matched_agent,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain("https://www.allrecipes.com/recipe/123").unwrap(),
            "www.allrecipes.com"
        );
    }

    #[test]
    fn test_extract_domain_invalid() {
        assert!(extract_domain("not-a-url").is_err());
    }

    #[test]
    fn test_robots_url() {
        assert_eq!(
            robots_url("https://example.com/path/page").unwrap(),
            "https://example.com/robots.txt"
        );
        assert_eq!(
            robots_url("http://example.com:8080/foo").unwrap(),
            "http://example.com:8080/robots.txt"
        );
    }

    #[test]
    fn test_parse_timestamp() {
        let ts = parse_timestamp("2026-02-11T10:30:00.000Z");
        assert!(ts.is_some());
    }

    #[test]
    fn test_is_within_ttl_recent() {
        let now = now_utc();
        assert!(is_within_ttl(&now));
    }

    #[test]
    fn test_is_within_ttl_old() {
        // A timestamp from 2020 should be expired
        assert!(!is_within_ttl("2020-01-01T00:00:00.000Z"));
    }
}
