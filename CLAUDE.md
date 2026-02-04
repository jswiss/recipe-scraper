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

- **Python 3.11+**: URL ingestion module (`src/url_ingestion/`)
- **requests**: HTTP client for fetching web pages
- **idna**: IDN/Punycode support for international domains
- **pytest**: Testing framework
- **ruff**: Linting and formatting

## Project Structure

```
src/
├── url_ingestion/          # URL validation, normalization, fetching
│   ├── __init__.py         # Public API: ingest_url(), validate_url()
│   ├── models.py           # FetchResult, FetchError, NormalizedURL
│   ├── validator.py        # URL syntax and protocol validation
│   ├── normalizer.py       # URL normalization (lowercase, IDN, ports)
│   └── fetcher.py          # HTTP fetching with error handling

tests/
├── unit/                   # Component-level tests
└── integration/            # End-to-end flow tests

specs/                      # Feature specifications
├── 001-url-ingestion/      # Current feature
│   ├── spec.md             # Requirements and user stories
│   ├── plan.md             # Implementation plan
│   ├── tasks.md            # Task breakdown
│   └── ...
```

## Commands

```bash
# Run tests
pytest

# Run linting
ruff check .

# Format code
ruff format .

# Install dependencies
pip install -e .
```

## Code Style Guidelines

### Python

- Use type hints for all function signatures
- Prefer `dataclass(frozen=True)` for immutable data structures
- Use `Union` types (or `|` syntax) for result types that can succeed or fail
- Return structured errors, never raise exceptions for expected failure cases
- Keep functions under 20 lines; extract helpers for complex logic

### Error Handling Pattern

```python
# Good: Return typed errors
def fetch(url: str) -> FetchSuccess | FetchError:
    if not url:
        return FetchError(ErrorType.VALIDATION, "No URL provided", url, None)
    ...

# Bad: Raise exceptions for expected cases
def fetch(url: str) -> str:
    if not url:
        raise ValueError("No URL provided")  # Don't do this
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

- 001-url-ingestion: URL validation, normalization, and fetching module

<!-- MANUAL ADDITIONS START -->
<!-- Add project-specific notes below this line -->
<!-- MANUAL ADDITIONS END -->
