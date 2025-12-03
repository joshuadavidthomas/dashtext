# Plan: Database Migration to TypeScript/Drizzle

## Goal

Move database handling from Rust (sqlx) to TypeScript (Drizzle ORM) with an API facade pattern, enabling future backend swapping for web/ElectricSQL.

## Architecture

```
src/lib/
  api/
    types.ts                    # DraftAPI interface + DraftData type
    index.ts                    # Backend detection + re-exports
    backends/
      tauri.ts                  # Drizzle implementation
  db/
    schema.ts                   # Drizzle schema
    index.ts                    # DB client init
    migrations/
      0000_*.sql                # Generated migration
drizzle.config.ts               # Drizzle Kit config

src-tauri/
  src/
    lib.rs                      # Simplified - just sql plugin setup
  Cargo.toml                    # Remove sqlx deps, add sql plugin
  capabilities/
    default.json                # Add sql permissions
```

## Migration Strategy

- **Schema ownership:** Drizzle generates SQL migration files
- **Migration execution:** Rust-side via `tauri-plugin-sql` using `include_str!()` 
- **Queries:** JS-side via Drizzle ORM through `tauri-plugin-sql`
- **Fresh database:** Starting clean, no existing data migration

---

## Tasks

### Task 1: Install Dependencies

**JS dependencies:**
```bash
bun add drizzle-orm @tauri-apps/plugin-sql
bun add -D drizzle-kit
```

**Rust dependencies (in src-tauri/):**
```bash
cargo add tauri-plugin-sql --features sqlite
```

**Verification:** `package.json` has drizzle deps, `Cargo.toml` has `tauri-plugin-sql`

---

### Task 2: Create Drizzle Schema and Config

**Create `src/lib/db/schema.ts`:**
```typescript
import { sqliteTable, integer, text, index } from 'drizzle-orm/sqlite-core';

export const drafts = sqliteTable('draft', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  content: text('content').notNull().default(''),
  createdAt: text('created_at').notNull(),
  modifiedAt: text('modified_at').notNull(),
}, (table) => [
  index('idx_draft_modified_at').on(table.modifiedAt),
]);

export type Draft = typeof drafts.$inferSelect;
export type NewDraft = typeof drafts.$inferInsert;
```

**Create `drizzle.config.ts`:**
```typescript
import { defineConfig } from 'drizzle-kit';

export default defineConfig({
  out: './src/lib/db/migrations',
  schema: './src/lib/db/schema.ts',
  dialect: 'sqlite',
});
```

**Generate migration:**
```bash
bun drizzle-kit generate
```

**Verification:** 
- `src/lib/db/schema.ts` exists with drafts table
- `drizzle.config.ts` exists
- `src/lib/db/migrations/` contains generated SQL file

---

### Task 3: Create DB Client

**Create `src/lib/db/index.ts`:**

Initialize tauri-plugin-sql connection and wrap with Drizzle's sqlite-proxy driver.

Key points:
- Use `Database.load('sqlite:dashtext.db')` from `@tauri-apps/plugin-sql`
- Wrap with `drizzle()` from `drizzle-orm/sqlite-proxy`
- Export `getDb()` async function that lazily initializes
- Export schema for use in queries

**Verification:** File exists, TypeScript compiles (`bun run check`)

---

### Task 4: Create API Facade Layer

**Create `src/lib/api/types.ts`:**
```typescript
export interface DraftData {
  id: number;
  content: string;
  created_at: string;
  modified_at: string;
}

export interface DraftAPI {
  list(): Promise<DraftData[]>;
  create(): Promise<DraftData>;
  get(id: number): Promise<DraftData | null>;
  save(id: number, content: string): Promise<DraftData>;
  delete(id: number): Promise<void>;
}
```

**Create `src/lib/api/backends/tauri.ts`:**

Implement `DraftAPI` interface using Drizzle queries:
- `list()`: SELECT all, ORDER BY modified_at DESC
- `create()`: INSERT with empty content, current timestamps
- `get(id)`: SELECT WHERE id
- `save(id, content)`: UPDATE content and modified_at
- `delete(id)`: DELETE WHERE id

Helper function to convert Drizzle row (camelCase) to DraftData (snake_case).

**Create `src/lib/api/index.ts`:**

Backend detection and re-exports:
- Check for `__TAURI_INTERNALS__` in window
- Dynamic import of tauri backend
- Re-export convenience functions: `listDrafts`, `createDraft`, `getDraft`, `saveDraft`, `deleteDraft`

**Update `src/lib/api/drafts.ts`:**

Simple re-export from index.ts for backward compatibility:
```typescript
export * from './index';
```

**Verification:** 
- All files exist
- TypeScript compiles
- `drafts.svelte.ts` imports still work (no changes needed to that file)

---

### Task 5: Update Tauri Rust Backend

**Update `src-tauri/capabilities/default.json`:**

Add SQL plugin permissions:
```json
{
  "permissions": [
    "core:default",
    "opener:default",
    "sql:default",
    "sql:allow-execute"
  ]
}
```

**Update `src-tauri/src/lib.rs`:**

Replace current implementation with:
- Import `tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind}`
- Define migrations vector with `include_str!()` pointing to Drizzle's generated SQL
- Register sql plugin with migrations
- Remove all `#[tauri::command]` functions
- Remove `.invoke_handler()`
- Remove db module import and Database state

**Update `src-tauri/Cargo.toml`:**

Remove dependencies:
- `sqlx`
- `tokio`
- `directories`
- `jiff`
- `anyhow`

Keep:
- `tauri`
- `tauri-plugin-opener`
- `tauri-plugin-sql` (with sqlite feature)
- `serde`
- `serde_json`

**Delete files:**
- `src-tauri/src/db.rs`
- `src-tauri/migrations/` directory

**Verification:**
- `cargo check` passes in src-tauri/
- No Rust compilation errors

---

### Task 6: Integration Testing

**Manual verification steps:**

1. Delete existing database file (fresh start)
2. Run `bun run tauri dev`
3. App launches without errors
4. Create a new draft - verify it appears in sidebar
5. Edit draft content - verify autosave works
6. Close and reopen app - verify content persisted
7. Create multiple drafts - verify list ordering (most recent first)
8. Delete a draft - verify removal
9. Check database location is correct (platform-specific app config dir)

**Verification:** All manual tests pass

---

## Files Changed Summary

**New files:**
- `src/lib/db/schema.ts`
- `src/lib/db/index.ts`
- `src/lib/db/migrations/*.sql`
- `src/lib/api/types.ts`
- `src/lib/api/backends/tauri.ts`
- `src/lib/api/index.ts`
- `drizzle.config.ts`

**Modified files:**
- `package.json`
- `src/lib/api/drafts.ts`
- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/capabilities/default.json`

**Deleted files:**
- `src-tauri/src/db.rs`
- `src-tauri/migrations/20241202000000_create_draft.sql`

**Unchanged files:**
- `src/lib/stores/drafts.svelte.ts` (imports from `$lib/api/drafts` which re-exports)
