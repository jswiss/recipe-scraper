# Local First Compliance Check

Audit code for Local First Software principles compliance per project constitution.

## Trigger

Use when:
- Reviewing features that involve data storage
- Adding network functionality
- Before merging features that handle user data

## Local First Principles (Ink & Switch)

### The Seven Ideals

1. **No Spinners**: Local operations respond instantly
2. **Your Work Lives With You**: Data on device, not just in cloud
3. **Network Optional**: Works offline without degradation
4. **Seamless Collaboration**: Sync when online, no conflicts
5. **Longevity**: Data survives service shutdown
6. **Security & Privacy**: User controls their data
7. **User Ownership**: No vendor lock-in

## Compliance Checklist

### Data Storage

- [ ] **Primary storage is local** (filesystem, SQLite, IndexedDB)
- [ ] **No cloud-only data** - user data never exists only on remote server
- [ ] **Export functionality** - users can extract all their data
- [ ] **Import functionality** - users can restore from backup
- [ ] **Standard formats** - data stored in portable formats (JSON, SQLite, not proprietary)

### Offline Capability

- [ ] **Core features work offline** - no network required for main functionality
- [ ] **Graceful degradation** - clear feedback when network features unavailable
- [ ] **No blocking network calls** - UI never waits on network for local ops
- [ ] **Sync is background** - synchronization doesn't interrupt user workflow

### Performance

- [ ] **Instant local ops** - reads/writes complete in <100ms
- [ ] **No loading states for local data** - spinners only for network ops
- [ ] **Optimistic updates** - UI updates immediately, syncs later

### Privacy & Security

- [ ] **Data stays local by default** - sync is opt-in
- [ ] **E2E encryption for sync** - server can't read user data (if applicable)
- [ ] **No analytics without consent** - respect user privacy

### Sync (if applicable)

- [ ] **Conflict resolution** - handles concurrent edits (CRDTs preferred)
- [ ] **Last-write-wins documented** - if using LWW, document the tradeoff
- [ ] **Offline queue** - changes queue locally, sync when online
- [ ] **Sync status visible** - user knows if data is synced

## Anti-Patterns to Flag

```
❌ fetch() in render path
❌ Cloud database as primary storage (Firebase, Supabase direct)
❌ Loading spinner for local data
❌ Features disabled when offline
❌ Data only accessible through API
❌ No export functionality
❌ Proprietary data formats
```

## Good Patterns

```
✅ SQLite/IndexedDB as primary store
✅ Background sync worker
✅ Optimistic UI updates
✅ CRDTs for collaborative data
✅ JSON/CSV export
✅ Works in airplane mode
```

## Output

Provide:
1. **Compliance Score**: X/7 ideals met
2. **Issues Found**: List with severity (CRITICAL/HIGH/MEDIUM/LOW)
3. **Recommendations**: Specific fixes for each issue
4. **Constitution Reference**: Which principle is violated

## Example Output

```
## Local First Compliance: 5/7

### Issues

| Severity | Issue | Location | Recommendation |
|----------|-------|----------|----------------|
| CRITICAL | Data only in cloud | src/api.py:45 | Add local SQLite cache |
| HIGH | Blocking fetch on startup | src/main.py:12 | Load from local first, fetch in background |
| MEDIUM | No export | - | Add JSON export function |

### Constitution Violations

- Principle VI (Local First): "Data MUST be stored locally on the user's device as the primary copy"
```
