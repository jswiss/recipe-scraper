-- Migration 002: robots.txt cache for compliance checking
-- Stores fetched robots.txt content per domain with a TTL.

CREATE TABLE IF NOT EXISTS robots_cache (
    domain     TEXT PRIMARY KEY,
    raw_content TEXT NOT NULL DEFAULT '',
    fetched_at TEXT NOT NULL,
    status     TEXT NOT NULL CHECK (status IN ('ok', 'not_found', 'unreachable', 'oversized'))
);
