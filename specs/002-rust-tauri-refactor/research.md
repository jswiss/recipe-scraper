# Research: Rust/Tauri Backend Refactor

**Feature**: 002-rust-tauri-refactor
**Date**: 2026-02-04

## HTTP Client Selection

**Decision**: `reqwest`

**Rationale**:
- De facto standard async HTTP client in Rust ecosystem
- Built on `hyper` with ergonomic API
- Native async/await support with tokio runtime
- Supports: redirects (configurable max), timeouts, streaming responses, custom headers
- `reqwest::Client` is cheap to clone and reuse

**Alternatives Considered**:
- `ureq`: Blocking-only, doesn't fit async Tauri command pattern
- `hyper`: Too low-level for simple URL fetching; reqwest wraps hyper

**Configuration**:
```rust
reqwest::Client::builder()
    .redirect(reqwest::redirect::Policy::limited(5))
    .timeout(Duration::from_secs(30))
    .user_agent("RecipeScraper/1.0")
    .build()
```

## URL Parsing & Normalization

**Decision**: `url` crate + `idna` crate

**Rationale**:
- `url` crate is the Rust implementation of the WHATWG URL Standard
- Provides parsing, normalization, and percent-encoding
- `idna` crate handles Unicode domain names (IDN to Punycode conversion)
- Both crates are maintained by the servo project (Mozilla)

**Alternatives Considered**:
- Manual parsing with regex: Error-prone, doesn't handle edge cases
- `http::Uri`: Focused on HTTP protocol, not URL normalization

**Key APIs**:
```rust
url::Url::parse(input)?      // Parse and validate
url.scheme()                  // "https"
url.host_str()               // Domain
url.port_or_known_default()  // Port handling
idna::domain_to_ascii(host)  // IDN conversion
```

## Tauri v2 Command Patterns

**Decision**: Async commands with `Result<T, E>` return types

**Rationale**:
- Tauri v2 uses `#[tauri::command]` macro for exposing Rust functions
- Async commands return `Promise` to JavaScript, keeping UI responsive
- Commands can return `Result<T, E>` where E implements `serde::Serialize`
- Use `tauri::State<>` for shared resources (HTTP client)

**Pattern**:
```rust
#[tauri::command]
async fn ingest_url(url: String, client: State<'_, HttpClient>) -> Result<FetchSuccess, FetchError> {
    // Implementation
}
```

**Frontend Invocation**:
```typescript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<FetchSuccess>('ingest_url', { url: 'https://...' });
```

## Project Structure

**Decision**: Standard Tauri v2 layout with modular Rust backend

**Rationale**:
- Tauri v2 expects `src-tauri/` directory with Rust code
- Module organization mirrors the Python structure for familiarity
- Keep URL ingestion as a self-contained module

**Structure**:
```
src-tauri/
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json         # Tauri configuration
├── src/
│   ├── main.rs             # Tauri app entry point
│   ├── lib.rs              # Library root (optional)
│   └── url_ingestion/      # URL ingestion module
│       ├── mod.rs          # Module exports
│       ├── models.rs       # Data types (NormalizedUrl, FetchSuccess, FetchError)
│       ├── validator.rs    # URL validation
│       ├── normalizer.rs   # URL normalization
│       ├── fetcher.rs      # HTTP fetching
│       └── commands.rs     # Tauri command wrappers
```

## Error Handling

**Decision**: `thiserror` for error definitions, `serde` for serialization

**Rationale**:
- `thiserror` provides derive macro for clean error definitions
- Errors serialize to JSON for frontend consumption
- Enum-based errors map directly to Python's ErrorType

**Pattern**:
```rust
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum FetchError {
    #[error("No URL provided")]
    Validation { message: String, url: String },

    #[error("Network error: {message}")]
    Network { message: String, url: String },

    #[error("HTTP error: {status_code}")]
    Http { message: String, url: String, status_code: u16 },

    #[error("Invalid content type: {content_type}")]
    ContentType { message: String, url: String, content_type: String },

    #[error("Response too large")]
    Size { message: String, url: String, max_bytes: usize },
}
```

## Dependencies Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2.x | Desktop app framework |
| `reqwest` | 0.12+ | HTTP client |
| `url` | 2.x | URL parsing |
| `idna` | 1.x | IDN/Punycode |
| `serde` | 1.x | Serialization |
| `serde_json` | 1.x | JSON support |
| `thiserror` | 2.x | Error derives |
| `tokio` | 1.x | Async runtime (via Tauri) |

## Python Code Removal

**Decision**: Delete after Rust implementation verified

**Files to Remove**:
- `src/url_ingestion/` (entire Python module)
- `tests/` (Python tests)
- `pyproject.toml`
- Any `.py` files at root

**Verification Before Removal**:
1. All spec acceptance scenarios pass with Rust implementation
2. Manual testing of Tauri commands from frontend
3. No Python imports remain in codebase
