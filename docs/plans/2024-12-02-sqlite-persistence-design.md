# SQLite Persistence Design (v0.1.0)

## Overview

Add local-only SQLite persistence to Dashtext using sqlx. Every document is a "draft" - content autosaves on debounced changes, no explicit save action.

## Goals

- Simple, robust local persistence
- Rust persistence layer reusable in future web backend
- Frontend never touches SQL - only typed Tauri commands

## Schema

```sql
CREATE TABLE draft (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,  -- ISO 8601
    modified_at TEXT NOT NULL  -- ISO 8601
);
```

- INTEGER PRIMARY KEY (SQLite auto-increment)
- Timestamps as TEXT in ISO 8601 format
- No title column - sidebar derives display from first line of content

## Rust Structure

```
src-tauri/
├── migrations/
│   └── 20241202_001_create_draft.sql
└── src/
    ├── main.rs      # unchanged
    ├── lib.rs       # Tauri setup, manages Database state, commands
    └── db.rs        # Database wrapper, Draft struct + methods
```

### db.rs

- `Database`: newtype wrapper around `SqlitePool`, managed as Tauri state
- `Draft`: struct with methods (no trait for now):
  - `create(pool) -> Result<Draft>`
  - `find(pool, id) -> Result<Option<Draft>>`
  - `list(pool) -> Result<Vec<Draft>>`
  - `save(&self, pool) -> Result<Draft>`
  - `delete(pool, id) -> Result<()>`
- `init_db(path) -> Result<SqlitePool>`: creates pool, runs migrations

### Tauri Commands (lib.rs)

Thin wrappers with `draft_` prefix:

- `draft_create`
- `draft_get`
- `draft_list`
- `draft_save`
- `draft_delete`

## Frontend Structure

```
src/lib/
├── api/
│   └── drafts.ts        # Typed invoke wrappers
└── stores/
    └── drafts.svelte.ts # DraftsState context
```

### drafts.ts

```typescript
type Draft = {
  id: number;
  content: string;
  created_at: string;
  modified_at: string;
};

function listDrafts(): Promise<Draft[]>
function createDraft(): Promise<Draft>
function getDraft(id: number): Promise<Draft | null>
function saveDraft(id: number, content: string): Promise<Draft>
function deleteDraft(id: number): Promise<void>
```

### DraftsState

- Created at layout level via context
- Tracks: `drafts` list, `currentDraft`, loading states
- Debounced autosave (500-1000ms after typing stops)

## Behavior

1. **Launch**: Load draft list, select most recent (or create one if empty)
2. **Typing**: Updates local state immediately, triggers debounced save
3. **Switching**: Save current draft first, then load selected draft into editor
4. **Delete**: Remove from DB, select next draft (or create new if last)

## Migrations

Using `sqlx::migrate!("./migrations")`:

- Migrations run on app startup
- New migrations added as numbered SQL files
- Example: `20250115_002_add_draft_tags.sql`

## Future Considerations

- Draft struct/methods are Tauri-agnostic (reusable in Axum backend)
- Database wrapper pattern allows easy mocking for tests
- Can extract to shared crate when adding web backend
