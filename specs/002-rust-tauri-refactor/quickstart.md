# Quickstart: Rust/Tauri URL Ingestion

**Feature**: 002-rust-tauri-refactor
**Date**: 2026-02-04

## Prerequisites

- Rust 1.70+ (`rustup update stable`)
- Tauri CLI (`cargo install tauri-cli`)
- Node.js 18+ (for frontend)

## Installation

```bash
# Initialize Tauri in the project
cargo tauri init

# Dependencies are managed via Cargo.toml (created during implementation)
```

## Usage

### From Rust (Library)

```rust
use url_ingestion::{ingest_url, validate_url, FetchSuccess, FetchError};

// Validate and fetch a URL
let result = ingest_url("https://example.com/recipe").await;
match result {
    Ok(success) => {
        println!("Fetched {} bytes", success.html.len());
        println!("Status: {}", success.status_code);
    }
    Err(error) => {
        println!("Error: {}", error.message());
    }
}

// Validate only (no network request)
let normalized = validate_url("https://EXAMPLE.COM/Recipe/").await;
match normalized {
    Ok(url) => {
        println!("Normalized: {}", url.to_string()); // https://example.com/Recipe
    }
    Err(error) => {
        println!("Invalid URL: {}", error.message());
    }
}
```

### From TypeScript (Tauri Frontend)

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { FetchSuccess, NormalizedUrl, FetchError } from './contracts/tauri-commands';

// Fetch a URL
try {
  const result = await invoke<FetchSuccess>('ingest_url', {
    url: 'https://example.com/recipe'
  });
  console.log(`Got ${result.html.length} bytes`);
  console.log(`Content-Type: ${result.content_type}`);
  if (result.final_url) {
    console.log(`Redirected to: ${result.final_url}`);
  }
} catch (error) {
  const fetchError = error as FetchError;
  console.error(`${fetchError.error_type}: ${fetchError.message}`);
}

// Validate only
try {
  const normalized = await invoke<NormalizedUrl>('validate_url', {
    url: 'https://EXAMPLE.COM:443/recipe/'
  });
  // normalized.host === 'example.com'
  // normalized.port === null (default port removed)
  // normalized.path === '/recipe' (trailing slash removed)
} catch (error) {
  const fetchError = error as FetchError;
  console.error(`Invalid: ${fetchError.message}`);
}
```

## Error Handling

Errors are returned as structured objects with `error_type` discriminator:

```typescript
// TypeScript error handling
try {
  const result = await invoke<FetchSuccess>('ingest_url', { url });
} catch (error) {
  const e = error as FetchError;

  switch (e.error_type) {
    case 'validation':
      // Invalid URL format
      showUserError(`Invalid URL: ${e.message}`);
      break;
    case 'network':
      // DNS, timeout, connection error
      showUserError(`Network error: ${e.message}`);
      break;
    case 'http':
      // 4xx/5xx response
      const httpErr = e as HttpError;
      showUserError(`Server returned ${httpErr.status_code}`);
      break;
    case 'content_type':
      // Non-HTML response
      showUserError('URL does not return HTML');
      break;
    case 'size':
      // Response too large
      showUserError('Page is too large to process');
      break;
  }
}
```

## Normalization Examples

| Input | Normalized Output |
|-------|-------------------|
| `HTTPS://Example.COM/` | `https://example.com/` |
| `http://example.com:80/path` | `http://example.com/path` |
| `https://example.com:443/` | `https://example.com/` |
| `https://example.com/path/` | `https://example.com/path` |
| `https://münchen.de/` | `https://xn--mnchen-3ya.de/` |

## Configuration Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `TIMEOUT_SECONDS` | 30 | HTTP request timeout |
| `MAX_REDIRECTS` | 5 | Maximum redirect chain length |
| `MAX_SIZE_BYTES` | 10,485,760 | Maximum response size (10MB) |
| `USER_AGENT` | `"RecipeScraper/1.0"` | HTTP User-Agent header |

## Testing

```bash
# Run all tests
cargo test

# Run URL ingestion tests only
cargo test --package recipe-scraper --lib url_ingestion

# Run with logging
RUST_LOG=debug cargo test
```

## Common Issues

### "No URL provided"
Pass a non-empty URL string to the command.

### "Invalid URL: missing scheme"
Include `http://` or `https://` prefix.

### "Request timed out"
The server didn't respond within 30 seconds. Check network connectivity.

### "Expected HTML but received..."
The URL returns non-HTML content (JSON, images, etc.). Only HTML pages are supported.
