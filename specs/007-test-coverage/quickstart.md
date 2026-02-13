# Quickstart: Test Coverage

## Prerequisites

- Rust 1.77+ (stable toolchain)
- All existing tests passing: `cd src-tauri && cargo test`

## Running Tests

```bash
# Run all tests (existing + new)
cd src-tauri && cargo test

# Run only new integration tests
cd src-tauri && cargo test --test integration

# Run tests for a specific module
cd src-tauri && cargo test storage::backup
cd src-tauri && cargo test storage::change_log
cd src-tauri && cargo test url_ingestion::commands

# Run with output (see test names)
cd src-tauri && cargo test -- --nocapture
```

## Implementation Order

1. **Create test fixtures** (`tests/fixtures/`) — HTML and robots.txt files
2. **Add backup.rs tests** — highest priority gap (data safety)
3. **Add change_log.rs tests** — second priority gap (sync reliability)
4. **Add spec-aligned boundary tests** — fill gaps per coverage matrix
5. **Add integration pipeline tests** — end-to-end validation
6. **Add command wrapper tests** — verify Tauri API surface
7. **Verify all existing tests still pass** — regression check

## Key Patterns

### Creating an isolated test database
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;

    fn test_db() -> Database {
        Database::new_in_memory().unwrap()
    }
}
```

### Loading fixture data
```rust
// In integration tests (tests/ directory)
const JSONLD_HTML: &str = include_str!("fixtures/jsonld_recipe.html");
const MICRODATA_HTML: &str = include_str!("fixtures/microdata_recipe.html");
```

### Testing error cases
```rust
#[test]
fn restore_corrupted_backup_returns_error() {
    let db = test_db();
    let path = temp_backup_path("corrupted");
    std::fs::write(&path, b"not a database").unwrap();

    let result = restore_collection_from(&db, &path);
    assert!(result.is_err());

    // Clean up
    let _ = std::fs::remove_file(&path);
}
```

## Verification

After implementation, confirm:
- `cargo test` passes with 0 failures
- `cargo clippy` reports no warnings
- All spec acceptance scenarios have corresponding tests (see contracts/coverage-matrix.md)
