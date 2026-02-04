# Data Model: URL Ingestion

**Feature**: 001-url-ingestion
**Date**: 2026-02-04

## Entities

### URLInput

The raw string provided by the user.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `raw_url` | `str` | Non-empty | The original URL string as provided |

**Validation Rules**:
- Must not be empty or whitespace-only
- No length limit (validated during parsing)

---

### NormalizedURL

A validated and standardized URL ready for fetching.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `scheme` | `str` | "http" or "https" | Protocol (lowercase) |
| `host` | `str` | Valid hostname | Domain in ASCII (Punycode if IDN) |
| `port` | `int \| None` | 1-65535 or None | Only set if non-default |
| `path` | `str` | Starts with "/" | URL path (no trailing slash except root) |
| `query` | `str \| None` | Optional | Query string without leading "?" |
| `fragment` | `str \| None` | Optional | Fragment without leading "#" |

**Derived Properties**:
- `url`: Full reconstructed URL string

**Normalization Rules** (from FR-004):
1. Scheme and host converted to lowercase
2. Default ports removed (80 for HTTP, 443 for HTTPS)
3. Trailing slashes removed from path (except "/" root)
4. Unnecessarily percent-encoded chars decoded
5. IDN domains converted to Punycode (ASCII)

---

### FetchResult

The outcome of a URL ingestion operation. Always one of success or failure.

```
FetchResult = FetchSuccess | FetchError
```

---

### FetchSuccess

Successful fetch containing HTML content.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `url` | `NormalizedURL` | Required | The normalized URL that was fetched |
| `html` | `str` | Non-empty, ≤10MB | The HTML content of the page |
| `status_code` | `int` | 200-299 | HTTP status code |
| `content_type` | `str` | Contains "text/html" | Full Content-Type header value |
| `final_url` | `str \| None` | Optional | Final URL if redirected |

---

### FetchError

Structured error information for failed operations.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `error_type` | `ErrorType` | Required | Category of error |
| `message` | `str` | Non-empty | Human-readable error description |
| `url` | `str` | Required | The URL that was attempted |
| `details` | `dict \| None` | Optional | Additional context (status code, etc.) |

---

### ErrorType (Enumeration)

Categories of errors for SC-004 distinguishability.

| Value | Description | Example Trigger |
|-------|-------------|-----------------|
| `VALIDATION` | URL syntax or protocol invalid | "not-a-url", "ftp://..." |
| `NETWORK` | Connection failed | DNS failure, timeout, refused |
| `HTTP` | Server returned error status | 404, 500, 403 |
| `CONTENT_TYPE` | Response is not HTML | PDF, JSON, image |
| `SIZE` | Response exceeds 10MB limit | Large media file |

---

## State Transitions

URL ingestion is stateless - each call is independent. However, the logical flow is:

```
URLInput
    │
    ▼ (validate)
┌───────────────┐
│ Valid syntax? │──No──► FetchError(VALIDATION)
└───────────────┘
    │ Yes
    ▼ (normalize)
NormalizedURL
    │
    ▼ (fetch)
┌───────────────┐
│ Connected?    │──No──► FetchError(NETWORK)
└───────────────┘
    │ Yes
    ▼
┌───────────────┐
│ Status 2xx?   │──No──► FetchError(HTTP)
└───────────────┘
    │ Yes
    ▼
┌───────────────┐
│ HTML type?    │──No──► FetchError(CONTENT_TYPE)
└───────────────┘
    │ Yes
    ▼
┌───────────────┐
│ Size ≤10MB?   │──No──► FetchError(SIZE)
└───────────────┘
    │ Yes
    ▼
FetchSuccess
```

## Relationships

```
URLInput ──(1:1)──► NormalizedURL ──(1:1)──► FetchResult
                                                  │
                                          ┌───────┴───────┐
                                          │               │
                                    FetchSuccess    FetchError
                                          │               │
                                          │         ErrorType
                                    NormalizedURL
```
