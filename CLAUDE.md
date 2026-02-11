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
- Rust 1.77+ (stable toolchain) (003-recipe-extraction)
- N/A (extraction only; storage is separate feature) (003-recipe-extraction)
- Rust 1.77+ (stable toolchain) + serde 1.x, thiserror 2.x, tauri 2.x (all already in Cargo.toml; no new dependencies) (004-recipe-tagging)
- N/A (tagging is compute-only; persistence is a separate feature) (004-recipe-tagging)
- Rust 1.77+ (stable toolchain) + tauri 2.1.0, serde 1.0, thiserror 2, rusqlite 0.38 (NEW — `bundled` feature) (005-persistent-state)
- SQLite (local file, WAL mode) via rusqlite (005-persistent-state)

### Rust/Tauri Backend
- **Rust 1.77+**: Backend language (stable toolchain)
- **Tauri 2.x**: Desktop application framework
- **reqwest 0.12+**: Async HTTP client
- **url 2.x**: URL parsing (WHATWG standard)
- **idna 1.x**: IDN/Punycode support
- **serde 1.x**: Serialization
- **thiserror 2.x**: Error handling
- **tokio 1.x**: Async runtime

## Project Structure

```
src-tauri/                   # Rust/Tauri backend
├── Cargo.toml               # Rust dependencies
├── tauri.conf.json          # Tauri configuration
├── capabilities/            # Tauri permissions
└── src/
    ├── main.rs              # Tauri entry point
    ├── lib.rs               # Library root
    └── url_ingestion/       # URL ingestion module
        ├── mod.rs           # Module exports
        ├── models.rs        # Data types (NormalizedUrl, FetchSuccess, FetchError)
        ├── validator.rs     # URL validation
        ├── normalizer.rs    # URL normalization
        ├── fetcher.rs       # HTTP fetching
        └── commands.rs      # Tauri commands

dist/                        # Frontend build output
specs/                       # Feature specifications
├── 001-url-ingestion/       # Original Python implementation (archived)
├── 002-rust-tauri-refactor/ # Rust refactor (complete)
```

## Commands

```bash
# Build and Run
cd src-tauri
cargo build                  # Build Rust backend
cargo test                   # Run Rust tests
cargo tauri dev              # Run Tauri dev server
cargo tauri build            # Build release binary

# Code Quality
cargo clippy                 # Run linter
cargo fmt                    # Format code
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

### Tauri Command Pattern

```rust
// Commands accept State for shared resources
#[tauri::command]
pub async fn ingest_url(
    url: String,
    client: State<'_, HttpClient>
) -> Result<FetchSuccess, FetchError> {
    // validate -> normalize -> fetch
}

// Frontend invocation
const result = await invoke<FetchSuccess>('ingest_url', { url: '...' });
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

## Git Workflow


### Branch Strategy

- Create a new branch for each speckit feature: `###-feature-name` (e.g., `001-url-ingestion`)
- Branch from `main` for new features

### Commit Cadence

**IMPORTANT**: Automatically `git add` and `git commit` after each step without waiting for user to ask.

1. **After each speckit step** (AUTO-COMMIT):
   - `/speckit.specify` → auto-commit spec.md, checklists/
   - `/speckit.clarify` → auto-commit updated spec.md
   - `/speckit.plan` → auto-commit plan.md, research.md, data-model.md, contracts/, quickstart.md, CLAUDE.md
   - `/speckit.tasks` → auto-commit tasks.md, updated plan.md
   - `/speckit.implement` → auto-commit after each task completion

2. **During implementation** (AUTO-COMMIT):
   - Commit after completing each task in tasks.md
   - Commit after completing each phase

3. **Pull Requests**:
   - Create PR at the end of each phase in tasks.md
   - PR into `main` when feature is complete
   - **Limit PRs to ~500 lines of code** - if a phase exceeds this, split into multiple PRs

### Commit Message Format

```bash
# Speckit artifacts
docs: add feature specification for <feature>
docs: add implementation plan for <feature>
docs: add implementation tasks for <feature>

# Implementation
feat: implement <component/functionality>
fix: resolve <issue>
refactor: <description>
test: add tests for <component>
```

## Recent Changes
- 005-persistent-state: Added Rust 1.77+ (stable toolchain) + tauri 2.1.0, serde 1.0, thiserror 2, rusqlite 0.38 (NEW — `bundled` feature)
- 004-recipe-tagging: Added Rust 1.77+ (stable toolchain) + serde 1.x, thiserror 2.x, tauri 2.x (all already in Cargo.toml; no new dependencies)
- 003-recipe-extraction: Added Rust 1.77+ (stable toolchain)


<!-- MANUAL ADDITIONS START -->
<!-- Add project-specific notes below this line -->
<!-- MANUAL ADDITIONS END -->
