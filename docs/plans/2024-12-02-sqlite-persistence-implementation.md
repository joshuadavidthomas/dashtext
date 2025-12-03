# SQLite Persistence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add local-only SQLite persistence for drafts using sqlx, with autosave behavior.

**Architecture:** Rust db module (Database wrapper + Draft struct with methods) → Tauri commands (thin wrappers) → TypeScript API layer → Svelte context (DraftsState) → UI components.

**Tech Stack:** sqlx (sqlite, runtime-tokio), Tauri v2, Svelte 5 runes, TypeScript, runed (useDebounce)

**Existing Dependencies (no changes needed):**
- Frontend: `runed` (for useDebounce), `@tauri-apps/api` (for invoke)
- These are already in package.json

---

## Task 1: Add sqlx Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add sqlx, tokio, and directories dependencies**

Add these to the `[dependencies]` section in `src-tauri/Cargo.toml`:

```toml
anyhow = "1.0"
directories = "6.0"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
tokio = { version = "1.0", features = ["full"] }
```

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully (may take a while to fetch dependencies)

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add sqlx, directories, and tokio dependencies"
```

---

## Task 2: Create Migration File

**Files:**
- Create: `src-tauri/migrations/20241202000000_create_draft.sql`

**Step 1: Create migrations directory and schema file**

Create `src-tauri/migrations/20241202000000_create_draft.sql`:

```sql
-- Create draft table
CREATE TABLE IF NOT EXISTS draft (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

-- Index on modified_at for "most recent" queries
CREATE INDEX IF NOT EXISTS idx_draft_modified_at
ON draft(modified_at DESC);
```

**Step 2: Commit**

```bash
git add src-tauri/migrations/
git commit -m "feat: add draft table migration"
```

---

## Task 3: Create Database Module

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs` (add module declaration)

**Step 1: Create db.rs with Database wrapper and init function**

Create `src-tauri/src/db.rs`:

```rust
use anyhow::Result;
use directories::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;

/// Get the ProjectDirs instance for dashtext
pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "dashtext")
        .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))
}

/// Get the database path using XDG directories
/// Linux: ~/.local/share/dashtext/dashtext.db
/// macOS: ~/Library/Application Support/dashtext/dashtext.db
/// Windows: C:\Users\<User>\AppData\Roaming\dashtext\dashtext.db
pub fn get_db_path() -> Result<PathBuf> {
    Ok(get_project_dirs()?.data_dir().join("dashtext.db"))
}

#[derive(Clone)]
pub struct Database(SqlitePool);

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.0
    }
}

pub async fn init_db() -> Result<SqlitePool> {
    let db_path = get_db_path()?;

    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Create connection options with create_if_missing
    let db_url = format!("sqlite://{}", db_path.display());
    let connect_options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
```

**Step 2: Add module declaration to lib.rs**

At the top of `src-tauri/src/lib.rs`, add:

```rust
mod db;
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/lib.rs
git commit -m "feat: add database module with init and pool wrapper"
```

---

## Task 4: Add Draft Struct and Methods

**Files:**
- Modify: `src-tauri/src/db.rs`

**Step 1: Add Draft struct and all CRUD methods**

Add to `src-tauri/src/db.rs` after the `init_db` function:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Draft {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub modified_at: String,
}

impl Draft {
    /// Create a new empty draft
    pub async fn create(pool: &SqlitePool) -> Result<Draft> {
        let now = chrono_now();
        let result = sqlx::query_as::<_, Draft>(
            r#"
            INSERT INTO draft (content, created_at, modified_at)
            VALUES ('', ?1, ?1)
            RETURNING id, content, created_at, modified_at
            "#,
        )
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Find a draft by ID
    pub async fn find(pool: &SqlitePool, id: i64) -> Result<Option<Draft>> {
        let result = sqlx::query_as::<_, Draft>(
            "SELECT id, content, created_at, modified_at FROM draft WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// List all drafts, most recently modified first
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Draft>> {
        let results = sqlx::query_as::<_, Draft>(
            "SELECT id, content, created_at, modified_at FROM draft ORDER BY modified_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// Save this draft (update content and modified_at)
    pub async fn save(pool: &SqlitePool, id: i64, content: &str) -> Result<Draft> {
        let now = chrono_now();
        let result = sqlx::query_as::<_, Draft>(
            r#"
            UPDATE draft SET content = ?, modified_at = ?
            WHERE id = ?
            RETURNING id, content, created_at, modified_at
            "#,
        )
        .bind(content)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Delete a draft by ID
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM draft WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Get current time as ISO 8601 string
fn chrono_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Format as ISO 8601 (simplified - just use Unix timestamp for now, or add chrono later)
    format!("{}", now)
}
```

**Note:** The `chrono_now()` function uses Unix timestamps for simplicity. If you want proper ISO 8601 strings, add the `chrono` crate later.

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: add Draft struct with CRUD methods"
```

---

## Task 5: Wire Database into Tauri Setup

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Update lib.rs with database initialization and state management**

Replace the contents of `src-tauri/src/lib.rs` with:

```rust
mod db;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database asynchronously
            // Uses XDG directories via the `directories` crate
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match db::init_db().await {
                    Ok(pool) => {
                        app_handle.manage(Database::new(pool));
                        match db::get_db_path() {
                            Ok(path) => println!("Database initialized at: {}", path.display()),
                            Err(_) => println!("Database initialized"),
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize database: {}", e);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: initialize database on app startup"
```

---

## Task 6: Add Tauri Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add Tauri command functions**

Add these commands before the `run()` function in `src-tauri/src/lib.rs`:

```rust
use db::{Database, Draft};

#[tauri::command]
async fn draft_create(db: tauri::State<'_, Database>) -> Result<Draft, String> {
    Draft::create(db.pool()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_get(db: tauri::State<'_, Database>, id: i64) -> Result<Option<Draft>, String> {
    Draft::find(db.pool(), id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_list(db: tauri::State<'_, Database>) -> Result<Vec<Draft>, String> {
    Draft::list(db.pool()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_save(
    db: tauri::State<'_, Database>,
    id: i64,
    content: String,
) -> Result<Draft, String> {
    Draft::save(db.pool(), id, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_delete(db: tauri::State<'_, Database>, id: i64) -> Result<(), String> {
    Draft::delete(db.pool(), id)
        .await
        .map_err(|e| e.to_string())
}
```

**Step 2: Register commands in the invoke handler**

Update the `tauri::Builder` chain in `run()` to include the invoke handler:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // ... existing setup code ...
        })
        .invoke_handler(tauri::generate_handler![
            draft_create,
            draft_get,
            draft_list,
            draft_save,
            draft_delete,
        ])
        .run(tauri::generate_context!())
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands for draft CRUD"
```

---

## Task 7: Create TypeScript API Layer

**Files:**
- Create: `src/lib/api/drafts.ts`

**Step 1: Create the drafts API module**

Create `src/lib/api/drafts.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';

export type Draft = {
	id: number;
	content: string;
	created_at: string;
	modified_at: string;
};

export async function listDrafts(): Promise<Draft[]> {
	return invoke('draft_list');
}

export async function createDraft(): Promise<Draft> {
	return invoke('draft_create');
}

export async function getDraft(id: number): Promise<Draft | null> {
	return invoke('draft_get', { id });
}

export async function saveDraft(id: number, content: string): Promise<Draft> {
	return invoke('draft_save', { id, content });
}

export async function deleteDraft(id: number): Promise<void> {
	return invoke('draft_delete', { id });
}
```

**Step 2: Commit**

```bash
git add src/lib/api/drafts.ts
git commit -m "feat: add TypeScript API layer for drafts"
```

---

## Task 8: Create DraftsState Context

**Files:**
- Create: `src/lib/stores/drafts.svelte.ts`

**Step 1: Create the DraftsState context using Svelte 5 patterns**

This uses:
- Svelte 5's `createContext` for type-safe context (available since 5.40.0)
- `$state` class fields for reactive properties
- `useDebounce` from runed for debounced saves

Create `src/lib/stores/drafts.svelte.ts`:

```typescript
import { createContext } from 'svelte';
import { useDebounce } from 'runed';
import { listDrafts, createDraft, saveDraft, deleteDraft, type Draft } from '$lib/api/drafts';

/**
 * DraftsState - manages draft list and current draft with autosave
 */
export class DraftsState {
	drafts = $state<Draft[]>([]);
	currentDraft = $state<Draft | null>(null);
	isLoading = $state(false);
	error = $state('');

	// Debounced save using runed
	private debouncedSave = useDebounce(() => this.performSave(), () => 500);

	/**
	 * Initialize: load drafts, select most recent or create one if empty
	 */
	async init() {
		this.isLoading = true;
		this.error = '';

		try {
			this.drafts = await listDrafts();

			if (this.drafts.length === 0) {
				// Create initial draft if none exist
				const newDraft = await createDraft();
				this.drafts = [newDraft];
			}

			// Select most recent (first in list, since sorted by modified_at DESC)
			this.currentDraft = this.drafts[0];
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Select a draft by ID
	 */
	async selectDraft(id: number) {
		// Flush any pending save first
		this.debouncedSave.runScheduledNow();

		const draft = this.drafts.find((d) => d.id === id);
		if (draft) {
			this.currentDraft = draft;
		}
	}

	/**
	 * Update current draft content (triggers debounced save)
	 */
	updateContent(content: string) {
		if (!this.currentDraft) return;

		// Update local state immediately
		this.currentDraft = { ...this.currentDraft, content };

		// Update in drafts list too
		this.drafts = this.drafts.map((d) =>
			d.id === this.currentDraft!.id ? this.currentDraft! : d
		);

		// Trigger debounced save
		this.debouncedSave();
	}

	/**
	 * Create a new draft and select it
	 */
	async newDraft() {
		// Flush any pending save first
		this.debouncedSave.runScheduledNow();

		try {
			const draft = await createDraft();
			this.drafts = [draft, ...this.drafts];
			this.currentDraft = draft;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}

	/**
	 * Delete a draft by ID
	 */
	async removeDraft(id: number) {
		try {
			await deleteDraft(id);
			this.drafts = this.drafts.filter((d) => d.id !== id);

			// If we deleted the current draft, select another or create new
			if (this.currentDraft?.id === id) {
				if (this.drafts.length > 0) {
					this.currentDraft = this.drafts[0];
				} else {
					const newDraft = await createDraft();
					this.drafts = [newDraft];
					this.currentDraft = newDraft;
				}
			}
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}

	/**
	 * Actually save to database
	 */
	private async performSave() {
		if (!this.currentDraft) return;

		try {
			const updated = await saveDraft(this.currentDraft.id, this.currentDraft.content);
			// Update local state with server response (includes new modified_at)
			this.currentDraft = updated;
			this.drafts = this.drafts.map((d) => (d.id === updated.id ? updated : d));
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}
}

// Svelte 5 createContext returns [get, set] tuple
export const [getDraftsState, setDraftsState] = createContext<DraftsState>();

/**
 * Create DraftsState and set it in context
 * Call this in a parent component (e.g., +layout.svelte)
 */
export const createDraftsState = () => {
	const drafts = new DraftsState();
	setDraftsState(drafts);
	return drafts;
};
```

**Step 2: Commit**

```bash
git add src/lib/stores/drafts.svelte.ts
git commit -m "feat: add DraftsState context with autosave using runed"
```

---

## Task 9: Wire DraftsState into Layout

**Files:**
- Modify: `src/routes/+layout.svelte`

**Step 1: Create DraftsState context at layout level**

Update `src/routes/+layout.svelte`:

```svelte
<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { createDraftsState } from '$lib/stores/drafts.svelte';

	let { children } = $props();

	// Create drafts context at layout level
	const draftsState = createDraftsState();

	// Initialize on mount
	onMount(() => {
		draftsState.init();
	});
</script>

<div data-layout="root">
	{@render children()}
</div>
```

**Step 2: Commit**

```bash
git add src/routes/+layout.svelte
git commit -m "feat: create DraftsState context at layout level"
```

---

## Task 10: Update Sidebar with Draft List

**Files:**
- Modify: `src/lib/components/layout/Sidebar.svelte`

**Step 1: Display draft list in sidebar**

Update `src/lib/components/layout/Sidebar.svelte`:

```svelte
<script lang="ts">
	import { getDraftsState } from '$lib/stores/drafts.svelte';

	const draftsState = getDraftsState();

	/**
	 * Get display title from draft content (first line, truncated)
	 */
	function getDisplayTitle(content: string): string {
		const firstLine = content.split('\n')[0].trim();
		if (!firstLine) return 'Untitled';
		if (firstLine.length > 30) return firstLine.slice(0, 30) + '...';
		return firstLine;
	}
</script>

<aside data-layout="sidebar">
	<div class="sidebar-header">
		<button onclick={() => draftsState.newDraft()} class="new-draft-btn"> + New </button>
	</div>

	{#if draftsState.isLoading}
		<div class="loading">Loading...</div>
	{:else if draftsState.error}
		<div class="error">{draftsState.error}</div>
	{:else}
		<ul class="draft-list">
			{#each draftsState.drafts as draft (draft.id)}
				<li>
					<button
						class="draft-item"
						class:selected={draftsState.currentDraft?.id === draft.id}
						onclick={() => draftsState.selectDraft(draft.id)}
					>
						{getDisplayTitle(draft.content)}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</aside>

<style>
	.sidebar-header {
		padding: 0.5rem;
		border-bottom: 1px solid var(--border, #333);
	}

	.new-draft-btn {
		width: 100%;
		padding: 0.5rem;
		background: var(--accent, #4a9eff);
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}

	.draft-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.draft-item {
		width: 100%;
		padding: 0.75rem;
		text-align: left;
		background: none;
		border: none;
		border-bottom: 1px solid var(--border, #333);
		cursor: pointer;
		color: inherit;
	}

	.draft-item:hover {
		background: var(--hover, #2a2a2a);
	}

	.draft-item.selected {
		background: var(--selected, #3a3a3a);
	}

	.loading,
	.error {
		padding: 1rem;
		text-align: center;
	}

	.error {
		color: var(--error, #ff4a4a);
	}
</style>
```

**Step 2: Commit**

```bash
git add src/lib/components/layout/Sidebar.svelte
git commit -m "feat: display draft list in sidebar"
```

---

## Task 11: Connect Editor to DraftsState

**Files:**
- Modify: `src/lib/components/editor/Editor.svelte`

**Step 1: Update Editor to use current draft content**

This is the most complex change. The Editor needs to:
1. Load content from `draftsState.currentDraft` when it changes
2. Call `draftsState.updateContent()` when the user types

**Step 2: Full updated Editor.svelte**

```svelte
<script lang="ts">
	import { EditorView } from '@codemirror/view';
	import { EditorState } from '@codemirror/state';
	import { getCM } from '@replit/codemirror-vim';
	import { createExtensions } from './extensions';
	import { getEditorContext, type VimModeType } from './context.svelte';
	import { getDraftsState } from '$lib/stores/drafts.svelte';

	const editorState = getEditorContext();
	const draftsState = getDraftsState();

	// Store editor view reference for external updates
	let view: EditorView | null = null;

	// Map vim mode strings to our type
	function mapVimMode(mode: string, subMode?: string): VimModeType {
		if (mode === 'visual') {
			if (subMode === 'linewise') return 'visual-line';
			if (subMode === 'blockwise') return 'visual-block';
			return 'visual';
		}
		switch (mode) {
			case 'normal':
				return 'normal';
			case 'insert':
				return 'insert';
			case 'replace':
				return 'replace';
			default:
				return 'normal';
		}
	}

	// Update cursor position from editor state
	function updateCursorPosition(v: EditorView) {
		const pos = v.state.selection.main.head;
		const line = v.state.doc.lineAt(pos);
		editorState.setCursorPosition(line.number, pos - line.from + 1);
		editorState.setTotalLines(v.state.doc.lines);
	}

	// Vim mode change handler reference for cleanup
	let vimModeChangeHandler: ((event: { mode: string }) => void) | null = null;

	// Setup vim mode change listener
	function setupVimModeListener(v: EditorView) {
		const cm = getCM(v);
		if (!cm) return;

		vimModeChangeHandler = (event: { mode: string; subMode?: string }) => {
			editorState.setVimMode(mapVimMode(event.mode, event.subMode));
		};

		cm.on('vim-mode-change', vimModeChangeHandler);
	}

	// Cleanup vim mode listener
	function cleanupVimModeListener(v: EditorView) {
		if (!vimModeChangeHandler) return;
		const cm = getCM(v);
		if (!cm) return;

		cm.off('vim-mode-change', vimModeChangeHandler);
		vimModeChangeHandler = null;
	}

	// Track if we're currently syncing from drafts to avoid loops
	let isSyncingFromDraft = false;

	// Action for editor initialization - runs once on mount
	function initEditor(container: HTMLDivElement) {
		const state = EditorState.create({
			doc: draftsState.currentDraft?.content ?? '',
			extensions: [
				...createExtensions(),
				// DOM event handlers
				EditorView.domEventHandlers({
					keydown: (event, v) => {
						// Trap Tab in all modes except normal (allow focus navigation in normal)
						if (event.key === 'Tab' && editorState.vimMode !== 'normal') {
							event.preventDefault();
							if (editorState.vimMode === 'insert') {
								v.dispatch(v.state.replaceSelection('\t'));
							}
							return true;
						}
						return false;
					},
					focus: () => {
						editorState.setFocused(true);
					},
					blur: () => {
						editorState.setFocused(false);
					}
				}),
				// Update listener to sync content and cursor to context
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !isSyncingFromDraft) {
						const content = update.state.doc.toString();
						editorState.setContent(content);
						draftsState.updateContent(content);
					}
					if (update.selectionSet || update.docChanged) {
						updateCursorPosition(update.view);
					}
				})
			]
		});

		view = new EditorView({ state, parent: container });
		view.focus();
		editorState.setFocused(true);

		// Set initial vim mode state and setup listener
		editorState.setVimMode('normal');
		setupVimModeListener(view);

		return {
			destroy() {
				if (view) {
					cleanupVimModeListener(view);
					view.destroy();
					view = null;
				}
			}
		};
	}

	// Sync editor content when current draft changes
	$effect(() => {
		const draft = draftsState.currentDraft;
		if (draft && view) {
			const currentContent = view.state.doc.toString();
			if (currentContent !== draft.content) {
				isSyncingFromDraft = true;
				view.dispatch({
					changes: {
						from: 0,
						to: view.state.doc.length,
						insert: draft.content
					}
				});
				isSyncingFromDraft = false;
				// Also update editor state
				editorState.setContent(draft.content);
			}
		}
	});
</script>

<div use:initEditor class="editor-container" role="textbox" aria-label="Text editor"></div>

<style>
	.editor-container {
		width: 100%;
		height: 100%;
	}

	.editor-container :global(.cm-editor) {
		height: 100%;
	}
</style>
```

**Step 3: Commit**

```bash
git add src/lib/components/editor/Editor.svelte
git commit -m "feat: connect editor to DraftsState with autosave"
```

---

## Task 12: Manual Testing Checklist

> **Note:** This task is for manual verification by the user. The agent should NOT attempt to run the dev server.

**User Action Required:** Run `bun run tauri dev` and verify the following:

### Test Checklist

- [ ] **Fresh launch creates initial draft** - On first launch, app creates one empty draft automatically
- [ ] **Typing autosaves** - Type some text, wait 500ms, check that it persists (quit and relaunch to verify)
- [ ] **Create new draft** - Click "+ New" in sidebar, new draft appears and is selected
- [ ] **Switch between drafts** - Click different drafts in sidebar, editor content updates
- [ ] **Delete a draft** - Delete a draft (if delete UI exists), verify list updates correctly
- [ ] **Persistence across restarts** - Quit app, relaunch, all drafts should be present
- [ ] **Most recent selected on launch** - After restart, most recently modified draft is selected

### If issues are found:

Document the issue and work through debugging. Common issues:
- Database not initialized (check console for errors)
- Context not available (check component hierarchy)
- Sync loop (editor ↔ draftsState fighting)

### Final commit after testing passes

```bash
git add -A
git commit -m "feat: complete SQLite persistence for drafts (v0.1.0)"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add sqlx dependencies | `Cargo.toml` |
| 2 | Create migration | `migrations/` |
| 3 | Database module | `db.rs`, `lib.rs` |
| 4 | Draft struct + methods | `db.rs` |
| 5 | Tauri setup | `lib.rs` |
| 6 | Tauri commands | `lib.rs` |
| 7 | TypeScript API | `api/drafts.ts` |
| 8 | DraftsState context | `stores/drafts.svelte.ts` |
| 9 | Layout integration | `+layout.svelte` |
| 10 | Sidebar draft list | `Sidebar.svelte` |
| 11 | Editor integration | `Editor.svelte` |
| 12 | Manual testing | User verification |

## Key Svelte 5 Patterns Used

- **`createContext`** (Svelte 5.40+): Returns `[get, set]` tuple for type-safe context
- **`$state` class fields**: Reactive properties in classes
- **`$effect`**: For syncing editor content when draft changes
- **`useDebounce` from runed**: For debounced autosave (already in dependencies)
- **`onMount`**: For one-time initialization
