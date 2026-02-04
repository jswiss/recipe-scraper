# Go Module Scaffold

Generate a new Go module following project constitution principles.

## Trigger

Use when creating a new Go package or module in the project.

## Arguments

- `name`: Module/package name (required)
- `purpose`: Brief description of what this module does

## Constitution-Compliant Structure

```
src/<name>/
├── <name>.go           # Main entry point, public API
├── <name>_test.go      # Tests alongside implementation
├── types.go            # Data structures (if needed)
├── errors.go           # Error types (if needed)
└── internal/           # Private implementation details (if needed)
```

## Generated Code Patterns

### Main Entry Point (`<name>.go`)

```go
// Package <name> <purpose>.
package <name>

// <Name> is the main entry point for <purpose>.
// Returns a result or an error with context.
func <Name>(input string) (Result, error) {
    if input == "" {
        return Result{}, ErrEmptyInput
    }

    // Implementation here
    return Result{}, nil
}
```

### Error Types (`errors.go`)

```go
package <name>

import "errors"

var (
    // ErrEmptyInput is returned when input is empty or whitespace.
    ErrEmptyInput = errors.New("<name>: empty input")

    // ErrInvalidFormat is returned when input format is invalid.
    ErrInvalidFormat = errors.New("<name>: invalid format")
)

// Error wraps an underlying error with <name> context.
type Error struct {
    Op  string // Operation that failed
    Err error  // Underlying error
}

func (e *Error) Error() string {
    return "<name>: " + e.Op + ": " + e.Err.Error()
}

func (e *Error) Unwrap() error {
    return e.Err
}
```

### Result Types (`types.go`)

```go
package <name>

// Result represents the outcome of a <name> operation.
type Result struct {
    // Success fields
    Data    string
    Status  int

    // Metadata
    Source  string
}

// IsSuccess returns true if the result represents success.
func (r Result) IsSuccess() bool {
    return r.Status >= 200 && r.Status < 300
}
```

### Test File (`<name>_test.go`)

```go
package <name>_test

import (
    "testing"

    "<module>/<name>"
)

func Test<Name>_ValidInput(t *testing.T) {
    result, err := <name>.<Name>("valid-input")
    if err != nil {
        t.Fatalf("unexpected error: %v", err)
    }
    if !result.IsSuccess() {
        t.Errorf("expected success, got status %d", result.Status)
    }
}

func Test<Name>_EmptyInput(t *testing.T) {
    _, err := <name>.<Name>("")
    if err == nil {
        t.Fatal("expected error for empty input")
    }
    if err != <name>.ErrEmptyInput {
        t.Errorf("expected ErrEmptyInput, got %v", err)
    }
}
```

## Principles Applied

1. **Readable**: Descriptive names, clear package comment
2. **Simple**: Flat structure, no unnecessary abstraction
3. **AHA**: No interfaces until pattern emerges
4. **Minimal Deps**: stdlib only unless justified
5. **Local First**: No network in core logic; network code isolated

## Output

Generate:
1. Directory structure
2. All files with placeholder implementations
3. Test file with example tests
4. README.md with usage examples (optional)
