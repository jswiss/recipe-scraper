# Data Model: Rust/Tauri Backend Refactor

**Feature**: 002-rust-tauri-refactor
**Date**: 2026-02-04

## Entities

### ErrorType (Enum)

Categories of errors that can occur during URL ingestion.

| Variant | Serialized Value | Description |
|---------|------------------|-------------|
| Validation | `"validation"` | URL syntax or protocol errors |
| Network | `"network"` | DNS, timeout, connection failures |
| Http | `"http"` | HTTP 4xx/5xx responses |
| ContentType | `"content_type"` | Non-HTML responses |
| Size | `"size"` | Response exceeds 10MB limit |

### NormalizedUrl (Struct)

A validated and normalized URL ready for fetching.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| scheme | `String` | Protocol | `"http"` or `"https"` only |
| host | `String` | Domain in ASCII | Punycode if IDN |
| port | `Option<u16>` | Port number | `None` if default (80/443) |
| path | `String` | URL path | Starts with `"/"` |
| query | `Option<String>` | Query string | Without leading `"?"` |
| fragment | `Option<String>` | Fragment | Without leading `"#"` |

**Derived**: `url() -> String` - Reconstructs full URL string

### FetchSuccess (Struct)

Successful fetch result containing HTML content.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| url | `NormalizedUrl` | The normalized URL fetched | Required |
| html | `String` | HTML content | UTF-8 decoded |
| status_code | `u16` | HTTP status | 200-299 |
| content_type | `String` | Content-Type header | Must start with `text/html` |
| final_url | `Option<String>` | Final URL if redirected | `None` if no redirect |

### FetchError (Enum)

Structured error for failed operations. Each variant includes contextual data.

| Variant | Fields | When Used |
|---------|--------|-----------|
| Validation | `message`, `url` | Invalid URL syntax/protocol |
| Network | `message`, `url`, `details` | DNS/timeout/connection errors |
| Http | `message`, `url`, `status_code` | HTTP 4xx/5xx responses |
| ContentType | `message`, `url`, `content_type` | Non-HTML responses |
| Size | `message`, `url`, `max_bytes` | Response > 10MB |

### FetchResult (Type Alias)

```rust
pub type FetchResult = Result<FetchSuccess, FetchError>;
```

## Relationships

```
                    ┌─────────────────┐
                    │  ingest_url()   │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   validate()    │──────► FetchError::Validation
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  normalize()    │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │    fetch()      │──────► FetchError::{Network,Http,ContentType,Size}
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  FetchSuccess   │
                    └─────────────────┘
```

## Validation Rules

### URL Validation
- Scheme must be `http` or `https` (case-insensitive check, stored lowercase)
- Host must be present and non-empty
- Host may contain IDN characters (normalized to Punycode)

### Normalization Rules
1. Scheme lowercased
2. Host lowercased and converted to Punycode if IDN
3. Default ports removed (80 for http, 443 for https)
4. Path must start with `/` (default to `/` if empty)
5. Trailing slashes removed from path (except root `/`)
6. Percent-encoding normalized (decode safe characters)

### Fetch Constraints
- Timeout: 30 seconds
- Max redirects: 5
- Max response size: 10MB (10,485,760 bytes)
- Content-Type must start with `text/html`
- User-Agent: `RecipeScraper/1.0`

## Serialization

All types serialize to JSON using serde:

```json
// FetchSuccess
{
  "url": {
    "scheme": "https",
    "host": "example.com",
    "port": null,
    "path": "/recipe",
    "query": null,
    "fragment": null
  },
  "html": "<!DOCTYPE html>...",
  "status_code": 200,
  "content_type": "text/html; charset=utf-8",
  "final_url": null
}

// FetchError (tagged enum)
{
  "error_type": "validation",
  "message": "No URL provided",
  "url": ""
}

{
  "error_type": "http",
  "message": "Page not found (404)",
  "url": "https://example.com/missing",
  "status_code": 404
}
```
