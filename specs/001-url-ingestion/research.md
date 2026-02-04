# Research: URL Ingestion

**Feature**: 001-url-ingestion
**Date**: 2026-02-04

## Technology Decisions

### 1. HTTP Client Library

**Decision**: `requests` library

**Rationale**:
- De facto standard for HTTP in Python with 50k+ GitHub stars
- Simple, readable API aligned with Constitution Principle I
- Built-in redirect following, timeout handling, and streaming support
- Minimal transitive dependencies (urllib3, certifi, charset-normalizer, idna)
- Well-documented edge case handling for Content-Type, encoding detection

**Alternatives Considered**:
- `httpx`: More modern, async-capable, but adds complexity we don't need for sync-only use
- `urllib.request` (stdlib): Lower-level, requires more boilerplate for redirects/timeouts
- `aiohttp`: Async-only, overkill for single-URL sequential processing
- LangChain `WebBaseLoader`: Adds heavy LangChain dependency for simple fetch task

### 2. URL Validation Approach

**Decision**: `urllib.parse` (stdlib) + custom protocol validation

**Rationale**:
- Standard library means zero additional dependencies (Principle III)
- `urlparse()` handles RFC 3986 parsing reliably
- Protocol check is trivial (~5 lines) - no need for external validator
- Consistent with Python ecosystem conventions

**Alternatives Considered**:
- `validators` package: Adds dependency for trivial functionality
- `pydantic.HttpUrl`: Would require Pydantic dependency just for URL validation
- `furl`: More powerful URL manipulation, but unnecessary for our needs

### 3. IDN/Punycode Handling

**Decision**: `idna` library (already a `requests` dependency)

**Rationale**:
- Already installed as transitive dependency of `requests`
- Implements IDNA 2008 standard (RFC 5891-5895)
- Simple API: `idna.encode(domain)` returns ASCII-compatible bytes
- No additional dependency burden

**Alternatives Considered**:
- `encodings.idna` (stdlib): Implements older IDNA 2003, has known issues with some domains
- Manual Punycode: Complex, error-prone, reinventing the wheel

### 4. Response Size Limiting

**Decision**: Streaming with `iter_content()` and byte counting

**Rationale**:
- Memory-efficient: doesn't load entire response before checking size
- Allows early termination when limit exceeded
- Built into `requests` - no additional dependencies
- Aligns with 10MB limit from spec clarifications

**Implementation Pattern**:
```python
content = b""
for chunk in response.iter_content(chunk_size=8192):
    content += chunk
    if len(content) > MAX_SIZE:
        raise ResponseTooLargeError(...)
```

### 5. Error Classification

**Decision**: Custom exception hierarchy with error types

**Rationale**:
- Clear separation: ValidationError, NetworkError, HttpError, ContentTypeError, SizeError
- Enables SC-004: users can distinguish error types from response alone
- Simple dataclass or NamedTuple for structured error info
- No external dependency needed

**Error Types**:
| Error Type | Trigger | HTTP Involved? |
|------------|---------|----------------|
| `ValidationError` | Invalid URL syntax, wrong protocol | No |
| `NetworkError` | DNS failure, connection timeout, connection refused | Yes (pre-response) |
| `HttpError` | 4xx/5xx status codes | Yes (response received) |
| `ContentTypeError` | Non-HTML Content-Type header | Yes (response received) |
| `SizeError` | Response exceeds 10MB | Yes (during streaming) |

### 6. Content-Type Validation

**Decision**: Check `Content-Type` header for `text/html`

**Rationale**:
- Header check is fast and doesn't require downloading full response
- Handles common variations: `text/html`, `text/html; charset=utf-8`
- Rejects PDFs, images, JSON early before wasting bandwidth

**Implementation Pattern**:
```python
content_type = response.headers.get("Content-Type", "")
if not content_type.lower().startswith("text/html"):
    raise ContentTypeError(...)
```

## Deferred Decisions

### JavaScript-Heavy Pages (Playwright)

**Status**: Deferred to future feature

**Rationale**:
- Most recipe sites serve static HTML with schema.org markup
- Adding Playwright would significantly increase dependency footprint
- Can be added as separate "enhanced fetcher" module if needed
- Current spec focuses on MVP: simple HTTP fetch

### Rate Limiting / Politeness

**Status**: Out of scope per spec assumptions

**Rationale**:
- Single-URL processing doesn't require rate limiting
- Future batch processing feature would add this capability
- robots.txt parsing could be separate module

## Dependencies Summary

| Dependency | Version | Purpose | Transitive? |
|------------|---------|---------|-------------|
| `requests` | >=2.28 | HTTP client | No (direct) |
| `idna` | >=3.0 | IDN/Punycode | Yes (via requests) |
| `pytest` | >=7.0 | Testing | Dev only |

**Total new runtime dependencies**: 1 (`requests`)
