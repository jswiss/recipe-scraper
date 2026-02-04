# Recipe Scraper - Development Guidelines

## Project Constitution

This project follows strict principles defined in `.specify/memory/constitution.md`. All code
contributions MUST comply with these principles.

### Core Principles (Non-Negotiable)

1. **Readable & Simple Code**: Optimize for the reader. Descriptive names, single-purpose
   functions, max 3 levels of nesting. Comments explain "why", not "what".

2. **AHA Programming**: Avoid Hasty Abstractions. Prefer duplication over wrong abstractions.
   Only abstract after seeing a pattern 3+ times with clear value.

3. **Minimal Dependencies**: Evaluate every dependency for maintenance burden, security
   surface, and bundle size. Standard library first. Document rationale for non-obvious deps.

4. **Accessibility First**: All UI must achieve WCAG 2.1 Level AA. Keyboard navigable,
   semantic HTML, proper ARIA usage. (N/A for backend-only features)

5. **Monorepo + Open Source**: Single repository for related code. Prefer self-hostable
   open source over cloud vendor lock-in (AWS/Azure/GCP).

6. **Local First**: User's device is primary data source. Full offline support. No loading
   spinners for local operations. Sync is secondary.

## Active Technologies

### Rust/Tauri Backend (002-rust-tauri-refactor)
- **Rust 1.70+**: Backend language (stable toolchain)
- **Tauri 2.x**: Desktop application framework
- **reqwest 0.12+**: Async HTTP client
- **url 2.x**: URL parsing (WHATWG standard)
- **idna 1.x**: IDN/Punycode support
- **serde 1.x**: Serialization
- **thiserror 2.x**: Error handling

### Python (to be removed after Rust refactor)
- **Python 3.11+**: URL ingestion module (`src/url_ingestion/`)
- **requests**: HTTP client for fetching web pages
- **idna**: IDN/Punycode support for international domains
- **pytest**: Testing framework
- **ruff**: Linting and formatting

## Project Structure

```
src-tauri/                   # Rust/Tauri backend (new)
├── Cargo.toml               # Rust dependencies
├── tauri.conf.json          # Tauri configuration
└── src/
    ├── main.rs              # Tauri entry point
    └── url_ingestion/       # URL ingestion module
        ├── mod.rs           # Module exports
        ├── models.rs        # Data types
        ├── validator.rs     # URL validation
        ├── normalizer.rs    # URL normalization
        ├── fetcher.rs       # HTTP fetching
        └── commands.rs      # Tauri commands

src/                         # Python (to be removed)
├── url_ingestion/           # URL validation, normalization, fetching

specs/                       # Feature specifications
├── 001-url-ingestion/       # Python implementation (complete)
├── 002-rust-tauri-refactor/ # Rust refactor (in progress)
```

## Commands

```bash
# Rust/Tauri
cargo build                  # Build Rust backend
cargo test                   # Run Rust tests
cargo tauri dev              # Run Tauri dev server
cargo tauri build            # Build release

# Python (deprecated)
pytest                       # Run tests
ruff check .                 # Run linting
ruff format .                # Format code
pip install -e .             # Install dependencies
```

## Code Style Guidelines

### Rust

- Use `Result<T, E>` for operations that can fail
- Prefer `thiserror` derive macros for custom errors
- Use `serde` for serialization with `#[serde(rename_all = "snake_case")]`
- Keep functions small and focused
- Use `async`/`await` for I/O operations

### Error Handling Pattern (Rust)

```rust
// Good: Return Result with typed error
pub async fn fetch(url: &str) -> Result<FetchSuccess, FetchError> {
    if url.is_empty() {
        return Err(FetchError::Validation {
            message: "No URL provided".into(),
            url: url.into(),
        });
    }
    // ...
}

// Tauri command wrapper
#[tauri::command]
pub async fn ingest_url(url: String) -> Result<FetchSuccess, FetchError> {
    fetch(&url).await
}
```

### Dependency Rules

Before adding a dependency, answer:
1. Can this be done with stdlib in <50 lines?
2. Is the package actively maintained (commits in last 6 months)?
3. What are its transitive dependencies?
4. Document the rationale in research.md or comments

## Local First Checklist

When building features, verify:
- [ ] Data is stored locally first (filesystem, SQLite, IndexedDB)
- [ ] Feature works fully offline
- [ ] No network requests block the UI
- [ ] User can export/backup their data
- [ ] No cloud service is required for core functionality

## Recent Changes

- 002-rust-tauri-refactor: Refactoring Python backend to Rust with Tauri v2
- 001-url-ingestion: URL validation, normalization, and fetching module (Python)

<!-- MANUAL ADDITIONS START -->
<!-- Add project-specific notes below this line -->
<!-- MANUAL ADDITIONS END -->
