# Route Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor from single-route SPA to SvelteKit file-based routing with shareable URLs and proper navigation.

**Architecture:** Route-based data loading via SvelteKit load functions, props-based Editor component, `(editor)` layout group for shell UI.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Tauri (client-only, ssr=false)

**Design Doc:** `docs/plans/2025-01-03-route-refactor-design.md`

---

## Task 1: Refactor Editor to Props-Based

**Files:**
- Modify: `src/lib/components/editor/Editor.svelte`

**Step 1: Update Editor props interface**

Replace the DraftsState context usage with explicit props:

```svelte
<script lang="ts">
	import { EditorView } from '@codemirror/view';
	import { EditorState } from '@codemirror/state';
	import { getCM } from '@replit/codemirror-vim';
	import { createExtensions } from './extensions';
	import { getEditorContext, type VimModeType } from './context.svelte';

	interface Props {
		initialContent?: string;
		onchange?: (content: string) => void;
	}

	let { initialContent = '', onchange }: Props = $props();

	const editorState = getEditorContext();

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

	// Action for editor initialization - runs once on mount
	function initEditor(container: HTMLDivElement) {
		const state = EditorState.create({
			doc: initialContent,
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
					if (update.docChanged) {
						const content = update.state.doc.toString();
						editorState.setContent(content);
						onchange?.(content);
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

**Step 2: Verify type check passes**

Run: `bun run check`
Expected: 0 errors (Editor is not currently mounted anywhere that would break)

**Step 3: Commit**

```bash
git add src/lib/components/editor/Editor.svelte
git commit -m "refactor(editor): change to props-based API"
```

---

## Task 2: Simplify drafts.svelte.ts

**Files:**
- Modify: `src/lib/stores/drafts.svelte.ts`

**Step 1: Remove DraftsState class, keep only Draft model**

Replace entire file with:

```ts
/**
 * Raw draft data shape from Tauri API
 */
export type DraftData = {
	id: number;
	content: string;
	created_at: string;
	modified_at: string;
};

/**
 * Draft - reactive draft model with derived presentation properties
 */
export class Draft {
	id: number;
	content = $state('');
	created_at: string;
	modified_at = $state('');

	title = $derived(this.content.split('\n')[0].trim() || 'Untitled');

	previewLines = $derived.by(() => {
		const lines = this.content.split('\n').slice(1);
		return lines.filter((line) => line.trim()).slice(0, 3);
	});

	formattedModifiedAt = $derived.by(() => {
		const value = this.modified_at;
		const asNumber = parseInt(value);
		const date =
			!isNaN(asNumber) && value === String(asNumber)
				? new Date(asNumber * 1000) // Unix timestamp (seconds)
				: new Date(value); // ISO/RFC 3339 string

		if (isNaN(date.getTime())) return 'Unknown date';

		return date.toLocaleString(undefined, {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	});

	constructor(data: DraftData) {
		this.id = data.id;
		this.content = data.content;
		this.created_at = data.created_at;
		this.modified_at = data.modified_at;
	}
}
```

**Step 2: Verify type check (will fail - expected)**

Run: `bun run check`
Expected: Errors in files still importing DraftsState (this is expected, we'll fix in subsequent tasks)

**Step 3: Commit**

```bash
git add src/lib/stores/drafts.svelte.ts
git commit -m "refactor(stores): remove DraftsState class, keep Draft model"
```

---

## Task 3: Create Root Route Redirect

**Files:**
- Modify: `src/routes/+layout.svelte`
- Create: `src/routes/+page.ts`
- Delete content from: `src/routes/+page.svelte`

**Step 1: Simplify root +layout.svelte**

Replace with minimal layout:

```svelte
<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

{@render children()}
```

**Step 2: Create root +page.ts with redirect**

```ts
import { redirect } from '@sveltejs/kit';

export function load() {
	redirect(307, '/drafts');
}
```

**Step 3: Create minimal +page.svelte placeholder**

```svelte
<!-- Redirect handled by +page.ts -->
```

**Step 4: Commit**

```bash
git add src/routes/+layout.svelte src/routes/+page.ts src/routes/+page.svelte
git commit -m "feat(routes): add root redirect to /drafts"
```

---

## Task 4: Create (editor) Layout Group

**Files:**
- Create: `src/routes/(editor)/+layout.svelte`
- Create: `src/routes/(editor)/+layout.ts`
- Move: `src/routes/WinBar.svelte` -> `src/routes/(editor)/WinBar.svelte`
- Move: `src/routes/StatusLine.svelte` -> `src/routes/(editor)/StatusLine.svelte`

**Step 1: Create (editor)/+layout.ts with draft list loading**

```ts
import { listDrafts } from '$lib/api';

export async function load() {
	const drafts = await listDrafts();
	return { drafts };
}
```

**Step 2: Create (editor)/+layout.svelte with shell UI**

```svelte
<script lang="ts">
	import { createEditorContext } from '$lib/components/editor';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';

	let { data, children } = $props();

	createEditorContext();
</script>

<Sidebar.Provider>
	<div data-layout="root">
		<WinBar />
		<Sidebar.Root collapsible="offcanvas">
			<Sidebar.Content class="gap-0">
				{#each data.drafts as draft (draft.id)}
					<a
						href="/drafts/{draft.id}"
						class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
					>
						<div class="truncate text-sm font-medium text-sidebar-foreground">
							{draft.title}
						</div>
						{#each draft.previewLines as line}
							<div class="truncate text-xs text-sidebar-foreground/60">
								{line}
							</div>
						{/each}
						{#if draft.content.trim()}
							<div class="text-xs text-sidebar-foreground/40">
								{draft.formattedModifiedAt}
							</div>
						{/if}
					</a>
				{/each}
			</Sidebar.Content>
		</Sidebar.Root>
		<main data-layout="main">
			{@render children()}
		</main>
		<aside data-layout="aside">
			<!-- Future: aside content -->
		</aside>
		<StatusLine />
	</div>
</Sidebar.Provider>
```

**Step 3: Move and update WinBar.svelte**

Move file to `src/routes/(editor)/WinBar.svelte` and update to use route-based navigation:

```svelte
<script lang="ts">
	import * as AppBar from '$lib/components/appbar';
	import { Button } from '$lib/components/ui/button/index.js';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';
	import { deleteDraft } from '$lib/api';
	import { page } from '$app/state';
	import { goto, invalidateAll } from '$app/navigation';
	import { Minus, PanelLeft, Plus, Square, Trash2, X } from '@lucide/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	const appWindow = getCurrentWindow();
	const sidebar = useSidebar();

	const currentDraftId = $derived(page.params.id ? Number(page.params.id) : null);

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		await appWindow.toggleMaximize();
	}

	async function close() {
		await appWindow.close();
	}

	function handleNew() {
		goto('/drafts/new');
	}

	async function handleDelete() {
		if (!currentDraftId) return;
		await deleteDraft(currentDraftId);
		await invalidateAll();
		goto('/drafts');
	}
</script>

<Tooltip.Provider delayDuration={300}>
	<AppBar.Root as="header" data-layout="menu-bar" data-tauri-drag-region class="gap-1 bg-[var(--cm-background-dark)] p-1">
			<AppBar.Section style="-webkit-app-region: no-drag;">
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar"
								size="icon-sm"
								onclick={() => sidebar.toggle()}
								aria-label="Toggle sidebar"
								aria-pressed={sidebar.open}
								class={sidebar.open ? 'text-[var(--cm-accent)]' : ''}
							>
								<PanelLeft class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">Toggle sidebar (Ctrl+B)</Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar"
								size="icon-sm"
								onclick={handleNew}
								aria-label="New draft"
							>
								<Plus class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">New draft (Ctrl+N)</Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar"
								size="icon-sm"
								onclick={handleDelete}
								disabled={!currentDraftId}
								aria-label="Delete draft"
							>
								<Trash2 class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">Delete draft</Tooltip.Content>
				</Tooltip.Root>
			</AppBar.Section>

			<AppBar.Section style="-webkit-app-region: no-drag;">
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar-minimize"
								size="icon-sm"
								onclick={minimize}
								aria-label="Minimize"
							>
								<Minus class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">Minimize</Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar-maximize"
								size="icon-sm"
								onclick={toggleMaximize}
								aria-label="Maximize"
							>
								<Square class="size-3" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">Maximize</Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar-close"
								size="icon-sm"
								onclick={close}
								aria-label="Close"
							>
								<X class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">Close</Tooltip.Content>
				</Tooltip.Root>
			</AppBar.Section>
	</AppBar.Root>
</Tooltip.Provider>
```

**Step 4: Move StatusLine.svelte**

Move `src/routes/StatusLine.svelte` to `src/routes/(editor)/StatusLine.svelte` (no changes needed - it only uses editor context).

**Step 5: Delete old files from src/routes/**

```bash
rm src/routes/WinBar.svelte src/routes/StatusLine.svelte
```

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(routes): create (editor) layout group with shell UI"
```

---

## Task 5: Create /drafts Route with Redirect

**Files:**
- Create: `src/routes/(editor)/drafts/+page.ts`
- Create: `src/routes/(editor)/drafts/+page.svelte`

**Step 1: Create drafts/+page.ts with smart redirect**

```ts
import { redirect } from '@sveltejs/kit';

export async function load({ parent }) {
	const { drafts } = await parent();

	if (drafts.length === 0) {
		redirect(307, '/drafts/new');
	}

	// Most recent is first (sorted by modified_at DESC)
	redirect(307, `/drafts/${drafts[0].id}`);
}
```

**Step 2: Create minimal drafts/+page.svelte**

```svelte
<!-- Redirect handled by +page.ts -->
```

**Step 3: Commit**

```bash
git add src/routes/\(editor\)/drafts/
git commit -m "feat(routes): add /drafts redirect to most recent or new"
```

---

## Task 6: Create /drafts/[id] Route

**Files:**
- Create: `src/routes/(editor)/drafts/[id]/+page.ts`
- Create: `src/routes/(editor)/drafts/[id]/+page.svelte`

**Step 1: Create [id]/+page.ts with draft loading**

```ts
import { error } from '@sveltejs/kit';
import { getDraft } from '$lib/api';

export async function load({ params }) {
	const id = Number(params.id);

	if (isNaN(id)) {
		error(400, 'Invalid draft ID');
	}

	const draft = await getDraft(id);

	if (!draft) {
		error(404, 'Draft not found');
	}

	return { draft };
}
```

**Step 2: Create [id]/+page.svelte with Editor**

```svelte
<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { saveDraft } from '$lib/api';
	import { beforeNavigate } from '$app/navigation';
	import { useDebounce } from 'runed';

	let { data } = $props();

	// Track pending content for autosave
	let pendingContent: string | null = null;

	// Debounced save
	const debouncedSave = useDebounce(
		async () => {
			if (pendingContent !== null) {
				await saveDraft(data.draft.id, pendingContent);
				pendingContent = null;
			}
		},
		() => 500
	);

	function handleChange(content: string) {
		pendingContent = content;
		debouncedSave();
	}

	// Flush pending saves before navigating away
	beforeNavigate(() => {
		debouncedSave.runScheduledNow();
	});
</script>

<Editor initialContent={data.draft.content} onchange={handleChange} />
```

**Step 3: Commit**

```bash
git add src/routes/\(editor\)/drafts/\[id\]/
git commit -m "feat(routes): add /drafts/[id] route with autosave"
```

---

## Task 7: Create /drafts/new Route

**Files:**
- Create: `src/routes/(editor)/drafts/new/+page.svelte`

**Step 1: Create new/+page.svelte with create-on-save behavior**

```svelte
<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { createDraft, saveDraft } from '$lib/api';
	import { goto, invalidateAll } from '$app/navigation';
	import { useDebounce } from 'runed';

	// Track if we've created the draft yet
	let draftId: number | null = null;
	let pendingContent: string | null = null;

	// Debounced save (only runs after draft is created)
	const debouncedSave = useDebounce(
		async () => {
			if (draftId !== null && pendingContent !== null) {
				await saveDraft(draftId, pendingContent);
				pendingContent = null;
			}
		},
		() => 500
	);

	async function handleChange(content: string) {
		// First change: create the draft
		if (draftId === null && content.trim()) {
			const draft = await createDraft();
			draftId = draft.id;
			await saveDraft(draftId, content);
			await invalidateAll(); // Refresh sidebar
			// Replace URL without adding history entry
			goto(`/drafts/${draftId}`, { replaceState: true });
			return;
		}

		// Subsequent changes: debounced save
		if (draftId !== null) {
			pendingContent = content;
			debouncedSave();
		}
	}
</script>

<Editor initialContent="" onchange={handleChange} />
```

**Step 2: Commit**

```bash
git add src/routes/\(editor\)/drafts/new/
git commit -m "feat(routes): add /drafts/new route with create-on-save"
```

---

## Task 8: Add Active State to Sidebar Links

**Files:**
- Modify: `src/routes/(editor)/+layout.svelte`

**Step 1: Update sidebar to highlight active draft**

```svelte
<script lang="ts">
	import { page } from '$app/state';
	import { createEditorContext } from '$lib/components/editor';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';

	let { data, children } = $props();

	createEditorContext();

	const currentDraftId = $derived(page.params.id ? Number(page.params.id) : null);
</script>

<Sidebar.Provider>
	<div data-layout="root">
		<WinBar />
		<Sidebar.Root collapsible="offcanvas">
			<Sidebar.Content class="gap-0">
				{#each data.drafts as draft (draft.id)}
					<a
						href="/drafts/{draft.id}"
						class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
						class:bg-sidebar-accent={currentDraftId === draft.id}
					>
						<div class="truncate text-sm font-medium text-sidebar-foreground">
							{draft.title}
						</div>
						{#each draft.previewLines as line}
							<div class="truncate text-xs text-sidebar-foreground/60">
								{line}
							</div>
						{/each}
						{#if draft.content.trim()}
							<div class="text-xs text-sidebar-foreground/40">
								{draft.formattedModifiedAt}
							</div>
						{/if}
					</a>
				{/each}
			</Sidebar.Content>
		</Sidebar.Root>
		<main data-layout="main">
			{@render children()}
		</main>
		<aside data-layout="aside">
			<!-- Future: aside content -->
		</aside>
		<StatusLine />
	</div>
</Sidebar.Provider>
```

**Step 2: Commit**

```bash
git add src/routes/\(editor\)/+layout.svelte
git commit -m "feat(routes): highlight active draft in sidebar"
```

---

## Task 9: Verify and Clean Up

**Step 1: Run type check**

Run: `bun run check`
Expected: 0 errors

**Step 2: Test in Tauri dev mode**

Run: `bun run tauri dev`

Verify:
- App opens to `/drafts/[id]` or `/drafts/new`
- Sidebar shows drafts
- Clicking draft navigates and updates URL
- Browser back/forward works
- New draft button goes to `/drafts/new`
- Typing in `/drafts/new` creates draft and updates URL
- Delete works and navigates to next draft
- Autosave works

**Step 3: Delete any remaining unused imports/files**

Check for any orphaned files or imports that reference old DraftsState usage.

**Step 4: Final commit**

```bash
git add -A
git commit -m "chore: clean up unused code after route refactor"
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Refactor Editor to props-based API |
| 2 | Simplify drafts.svelte.ts (keep Draft model only) |
| 3 | Create root route redirect |
| 4 | Create (editor) layout group with shell UI |
| 5 | Create /drafts redirect route |
| 6 | Create /drafts/[id] route with autosave |
| 7 | Create /drafts/new route with create-on-save |
| 8 | Add active state to sidebar links |
| 9 | Verify and clean up |
