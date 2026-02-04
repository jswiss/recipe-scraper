# Go Code Review

Review Go code for compliance with project constitution and Go best practices.

## Trigger

Use this skill when reviewing Go code or before committing Go changes.

## Review Checklist

### Constitution Compliance

**I. Readable & Simple Code**
- [ ] Function names are verbs describing what they do (`FetchURL`, `ValidateInput`)
- [ ] Variable names are descriptive (`userInput` not `ui`, `httpClient` not `c`)
- [ ] Functions do one thing (single responsibility)
- [ ] No more than 3 levels of nesting; use early returns
- [ ] Complex logic has comments explaining WHY

**II. AHA Programming**
- [ ] No interfaces with single implementation (premature abstraction)
- [ ] No generic utilities "for future use"
- [ ] Duplication is acceptable if abstraction isn't clearly better
- [ ] Helper functions only exist if used 3+ times

**III. Minimal Dependencies**
- [ ] Standard library used where possible (`net/http`, `encoding/json`, etc.)
- [ ] Each dependency justified in comments or docs
- [ ] No dependencies for <50 lines of code
- [ ] `go mod tidy` run to remove unused deps

### Go-Specific Best Practices

**Error Handling**
```go
// Good: Wrap errors with context
if err != nil {
    return fmt.Errorf("failed to fetch %s: %w", url, err)
}

// Bad: Naked error return
if err != nil {
    return err
}
```

**Struct Design**
```go
// Good: Small, focused structs
type FetchResult struct {
    URL     string
    Content []byte
    Status  int
}

// Bad: Kitchen sink struct
type Result struct {
    URL, Content, Status, Error, Timestamp, Headers, Cookies, ...
}
```

**Interface Design**
```go
// Good: Small interfaces, defined by consumer
type Fetcher interface {
    Fetch(url string) ([]byte, error)
}

// Bad: Large interfaces defined by implementer
type HTTPClient interface {
    Get, Post, Put, Delete, Head, Options, Patch, ...
}
```

**Concurrency**
```go
// Good: Explicit synchronization
var mu sync.Mutex
mu.Lock()
defer mu.Unlock()

// Bad: Shared state without protection
```

### Local First Compliance

- [ ] Data stored locally before any network sync
- [ ] Graceful degradation when offline
- [ ] No blocking network calls in hot paths
- [ ] User data exportable without network

## Output

Provide:
1. List of issues found (with line numbers)
2. Suggested fixes
3. Constitution principle violated (if any)
