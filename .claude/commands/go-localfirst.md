# Go Local First Patterns

Generate Go code following Local First Software principles.

## Trigger

Use when implementing data storage, caching, or sync features in Go.

## Local First Architecture for Go

### Storage Layer Pattern

```go
// storage.go - Local-first storage abstraction

package storage

import (
    "database/sql"
    "encoding/json"
    "os"
    "path/filepath"
    "sync"

    _ "modernc.org/sqlite" // Pure Go SQLite (no CGO)
)

// Store provides local-first data storage.
// Data is always written locally first, sync is secondary.
type Store struct {
    db     *sql.DB
    path   string
    mu     sync.RWMutex
    syncer Syncer // Optional, can be nil for offline-only
}

// New creates a local-first store at the given path.
// Works immediately, no network required.
func New(dataDir string) (*Store, error) {
    dbPath := filepath.Join(dataDir, "data.db")

    // Ensure directory exists
    if err := os.MkdirAll(dataDir, 0755); err != nil {
        return nil, fmt.Errorf("create data dir: %w", err)
    }

    db, err := sql.Open("sqlite", dbPath)
    if err != nil {
        return nil, fmt.Errorf("open database: %w", err)
    }

    store := &Store{db: db, path: dbPath}
    if err := store.migrate(); err != nil {
        return nil, fmt.Errorf("migrate: %w", err)
    }

    return store, nil
}

// Save writes data locally (instant) and queues for sync (background).
func (s *Store) Save(key string, data any) error {
    s.mu.Lock()
    defer s.mu.Unlock()

    // Marshal to JSON for portability
    bytes, err := json.Marshal(data)
    if err != nil {
        return fmt.Errorf("marshal: %w", err)
    }

    // Write locally first - this is instant
    _, err = s.db.Exec(`
        INSERT OR REPLACE INTO data (key, value, updated_at, synced)
        VALUES (?, ?, datetime('now'), 0)
    `, key, bytes)
    if err != nil {
        return fmt.Errorf("save local: %w", err)
    }

    // Queue for background sync (non-blocking)
    if s.syncer != nil {
        go s.syncer.Queue(key, bytes)
    }

    return nil
}

// Load reads from local storage (instant, no network).
func (s *Store) Load(key string, dest any) error {
    s.mu.RLock()
    defer s.mu.RUnlock()

    var bytes []byte
    err := s.db.QueryRow(`SELECT value FROM data WHERE key = ?`, key).Scan(&bytes)
    if err == sql.ErrNoRows {
        return ErrNotFound
    }
    if err != nil {
        return fmt.Errorf("load: %w", err)
    }

    return json.Unmarshal(bytes, dest)
}
```

### Sync Pattern (Background, Non-Blocking)

```go
// sync.go - Background synchronization

package storage

import (
    "context"
    "time"
)

// Syncer handles background sync to remote.
// Never blocks local operations.
type Syncer interface {
    Queue(key string, data []byte)
    Start(ctx context.Context)
}

// BackgroundSyncer syncs data in background goroutine.
type BackgroundSyncer struct {
    queue   chan syncItem
    remote  RemoteStore
    store   *Store
}

type syncItem struct {
    key  string
    data []byte
}

func NewBackgroundSyncer(store *Store, remote RemoteStore) *BackgroundSyncer {
    return &BackgroundSyncer{
        queue:  make(chan syncItem, 1000), // Buffer for offline queuing
        remote: remote,
        store:  store,
    }
}

// Queue adds item to sync queue (non-blocking).
func (s *BackgroundSyncer) Queue(key string, data []byte) {
    select {
    case s.queue <- syncItem{key, data}:
        // Queued successfully
    default:
        // Queue full, will retry on next save
        // This is fine - local data is safe
    }
}

// Start runs the background sync loop.
func (s *BackgroundSyncer) Start(ctx context.Context) {
    for {
        select {
        case <-ctx.Done():
            return
        case item := <-s.queue:
            // Try to sync, retry on failure
            for retries := 0; retries < 3; retries++ {
                if err := s.remote.Put(item.key, item.data); err != nil {
                    time.Sleep(time.Second * time.Duration(retries+1))
                    continue
                }
                // Mark as synced in local DB
                s.store.markSynced(item.key)
                break
            }
        }
    }
}
```

### Export/Import Pattern

```go
// export.go - User data portability

package storage

import (
    "encoding/json"
    "io"
    "os"
)

// Export writes all user data to a portable JSON file.
// Users can backup and restore their data independently.
func (s *Store) Export(w io.Writer) error {
    s.mu.RLock()
    defer s.mu.RUnlock()

    rows, err := s.db.Query(`SELECT key, value FROM data`)
    if err != nil {
        return fmt.Errorf("query: %w", err)
    }
    defer rows.Close()

    data := make(map[string]json.RawMessage)
    for rows.Next() {
        var key string
        var value []byte
        if err := rows.Scan(&key, &value); err != nil {
            return fmt.Errorf("scan: %w", err)
        }
        data[key] = value
    }

    enc := json.NewEncoder(w)
    enc.SetIndent("", "  ")
    return enc.Encode(data)
}

// Import restores user data from a backup file.
func (s *Store) Import(r io.Reader) error {
    s.mu.Lock()
    defer s.mu.Unlock()

    var data map[string]json.RawMessage
    if err := json.NewDecoder(r).Decode(&data); err != nil {
        return fmt.Errorf("decode: %w", err)
    }

    tx, err := s.db.Begin()
    if err != nil {
        return fmt.Errorf("begin tx: %w", err)
    }
    defer tx.Rollback()

    for key, value := range data {
        _, err := tx.Exec(`
            INSERT OR REPLACE INTO data (key, value, updated_at, synced)
            VALUES (?, ?, datetime('now'), 0)
        `, key, []byte(value))
        if err != nil {
            return fmt.Errorf("insert %s: %w", key, err)
        }
    }

    return tx.Commit()
}
```

## Recommended Dependencies

```go
// go.mod - Minimal deps for local-first Go

module myapp

go 1.21

require (
    modernc.org/sqlite v1.28.0  // Pure Go SQLite, no CGO
)
```

**Why `modernc.org/sqlite`**:
- Pure Go, no CGO required
- Cross-compiles easily
- Single binary deployment
- Constitution Principle III compliant (well-maintained, focused)

## Anti-Patterns to Avoid

```go
// ❌ Bad: Network in critical path
func GetRecipe(id string) (*Recipe, error) {
    resp, err := http.Get(apiURL + "/recipes/" + id)  // Blocks on network!
    ...
}

// ✅ Good: Local first, network in background
func GetRecipe(id string) (*Recipe, error) {
    // Try local first (instant)
    recipe, err := store.Load("recipe:" + id, &Recipe{})
    if err == nil {
        return recipe, nil
    }

    // Fall back to network only if not found locally
    // And cache the result
    ...
}
```

## Output

When using this skill, generate:
1. Storage implementation following these patterns
2. Background sync (if needed)
3. Export/Import functions
4. Tests for offline scenarios
