# shadcn-svelte Sidebar Adoption Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace custom sidebar implementation with shadcn-svelte Sidebar component, adapted to work within the existing CSS Grid layout (not shadcn's default fixed positioning approach).

**Current State:**
- Custom `UIState` class for sidebar visibility toggle
- Custom `Sidebar.svelte` component with manual show/hide via `{#if}`
- CSS Grid layout in `layout.css` with `data-layout` attributes
- Toggle button in `MenuBar.svelte` using custom state

**Target State:**
- shadcn-svelte `Sidebar.Provider` managing sidebar state
- `useSidebar()` hook for toggle functionality
- Sidebar integrated into CSS Grid (not fixed positioning)
- **No animations** - instant show/hide via `duration: 0ms`
- Tokyo Night theme integration

**Key Constraints:**
- This is a Tauri app with custom titlebar (MenuBar with drag region)
- **Keep the existing CSS Grid layout** - adapt sidebar to work within it
- No animations - all durations set to `0ms`
- Single sidebar for now (left side only)
- Use existing `data-layout` attribute convention

---

## Task 0: Add Missing Type Helpers to utils.ts

**Files:**
- Modify: `src/lib/utils.ts`

The shadcn-svelte sidebar components expect type helpers that are missing from utils.ts.

**Step 1: Add type helpers**

Add these type helpers to `src/lib/utils.ts` after the `cn` function:

```typescript
// biome-ignore lint/suspicious/noExplicitAny: Generic type constraint needs any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// biome-ignore lint/suspicious/noExplicitAny: Generic type constraint needs any
export type WithoutChildren<T> = T extends { children?: any }
	? Omit<T, "children">
	: T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
	ref?: U | null;
};
```

**Step 2: Verify build passes**

Run: `bun run check`

**Step 3: Commit**

```bash
git add src/lib/utils.ts
git commit -m "feat: add type helpers for shadcn components"
```

---

## Task 1: Add Duration Constants and Disable Animations

**Files:**
- Modify: `src/lib/components/ui/sidebar/constants.ts`
- Modify: `src/lib/components/ui/sidebar/sidebar.svelte`
- Modify: `src/lib/components/ui/sidebar/sidebar-provider.svelte`
- Modify: `src/lib/components/ui/sidebar/sidebar-group-label.svelte`
- Create: `src/lib/components/ui/sheet/constants.ts`
- Modify: `src/lib/components/ui/sheet/sheet-content.svelte`

**Step 1: Add duration constant to sidebar constants**

Update `src/lib/components/ui/sidebar/constants.ts`:

```typescript
export const SIDEBAR_COOKIE_NAME = "sidebar:state";
export const SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7;
export const SIDEBAR_DURATION = "0ms";
export const SIDEBAR_WIDTH = "16rem";
export const SIDEBAR_WIDTH_MOBILE = "18rem";
export const SIDEBAR_WIDTH_ICON = "3rem";
export const SIDEBAR_KEYBOARD_SHORTCUT = "b";
```

**Step 2: Update sidebar-provider.svelte to use duration**

In `src/lib/components/ui/sidebar/sidebar-provider.svelte`, import `SIDEBAR_DURATION` and add it to the style:

Change line 43 from:
```svelte
style="--sidebar-width: {SIDEBAR_WIDTH}; --sidebar-width-icon: {SIDEBAR_WIDTH_ICON}; {style}"
```
to:
```svelte
style="--sidebar-duration: {SIDEBAR_DURATION}; --sidebar-width: {SIDEBAR_WIDTH}; --sidebar-width-icon: {SIDEBAR_WIDTH_ICON}; {style}"
```

And add `SIDEBAR_DURATION` to the imports at the top.

**Step 3: Update sidebar.svelte to use duration variable**

In `src/lib/components/ui/sidebar/sidebar.svelte`:

Line 72, change:
```svelte
"w-(--sidebar-width) relative bg-transparent transition-[width] duration-200 ease-linear",
```
to:
```svelte
"w-(--sidebar-width) relative bg-transparent transition-[width] duration-(--sidebar-duration) ease-linear",
```

Line 83, change:
```svelte
"w-(--sidebar-width) fixed inset-y-0 z-10 hidden h-svh transition-[left,right,width] duration-200 ease-linear md:flex",
```
to:
```svelte
"w-(--sidebar-width) fixed inset-y-0 z-10 hidden h-svh transition-[left,right,width] duration-(--sidebar-duration) ease-linear md:flex",
```

**Step 4: Update sidebar-group-label.svelte to use duration variable**

In `src/lib/components/ui/sidebar/sidebar-group-label.svelte`, line 18, change:
```svelte
"text-sidebar-foreground/70 ring-sidebar-ring outline-hidden flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium transition-[margin,opacity] duration-200 ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0",
```
to:
```svelte
"text-sidebar-foreground/70 ring-sidebar-ring outline-hidden flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium transition-[margin,opacity] duration-(--sidebar-duration) ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0",
```

**Step 5: Create sheet constants file**

Create `src/lib/components/ui/sheet/constants.ts`:

```typescript
export const SHEET_CLOSE_DURATION = "0ms";
export const SHEET_OPEN_DURATION = "0ms";
```

**Step 6: Update sheet-content.svelte to use duration variables**

In `src/lib/components/ui/sheet/sheet-content.svelte`:

Add import at top of second script block:
```typescript
import { SHEET_CLOSE_DURATION, SHEET_OPEN_DURATION } from "./constants.js";
```

Change the `sheetVariants` base class (line 4) from:
```typescript
base: "bg-background data-[state=open]:animate-in data-[state=closed]:animate-out fixed z-50 flex flex-col gap-4 shadow-lg transition ease-in-out data-[state=closed]:duration-300 data-[state=open]:duration-500",
```
to:
```typescript
base: "bg-background data-[state=open]:animate-in data-[state=closed]:animate-out fixed z-50 flex flex-col gap-4 shadow-lg transition ease-in-out data-[state=closed]:duration-(--sheet-close-duration) data-[state=open]:duration-(--sheet-open-duration)",
```

Add style prop to `SheetPrimitive.Content` (around line 46-50):
```svelte
<SheetPrimitive.Content
	bind:ref
	data-slot="sheet-content"
	style="--sheet-close-duration: {SHEET_CLOSE_DURATION}; --sheet-open-duration: {SHEET_OPEN_DURATION};"
	class={cn(sheetVariants({ side }), className)}
	{...restProps}
>
```

Also need to remove the `portalProps` handling and use `SheetPrimitive.Portal` directly if it's causing issues with the style prop.

**Step 7: Verify build passes**

Run: `bun run check`

**Step 8: Commit**

```bash
git add src/lib/components/ui/sidebar/constants.ts \
        src/lib/components/ui/sidebar/sidebar.svelte \
        src/lib/components/ui/sidebar/sidebar-provider.svelte \
        src/lib/components/ui/sidebar/sidebar-group-label.svelte \
        src/lib/components/ui/sheet/constants.ts \
        src/lib/components/ui/sheet/sheet-content.svelte
git commit -m "chore: add duration constants and disable animations"
```

---

## Task 2: Adapt Sidebar for CSS Grid Layout

**Files:**
- Modify: `src/lib/components/ui/sidebar/sidebar.svelte`

The default shadcn sidebar uses fixed positioning. We need to adapt it to work within our CSS Grid layout using `data-layout` attributes.

**Step 1: Modify sidebar.svelte for grid layout**

Replace the desktop sidebar section (the `{:else}` block starting around line 58) with a grid-compatible version.

The key changes:
1. Remove fixed positioning classes
2. Add `data-layout="sidebar"` attribute
3. Keep the `data-state` and `data-collapsible` attributes for state tracking
4. Simplify the structure since we don't need the gap div trick for fixed positioning

Replace the `{:else}` block (lines 58-103) with:

```svelte
{:else}
	<aside
		bind:this={ref}
		data-layout="sidebar"
		data-state={sidebar.state}
		data-collapsible={sidebar.state === "collapsed" ? collapsible : ""}
		data-variant={variant}
		data-side={side}
		data-slot="sidebar"
		class={cn(
			"bg-sidebar text-sidebar-foreground hidden flex-col md:flex",
			"transition-[width] duration-(--sidebar-duration) ease-linear",
			"group-data-[collapsible=offcanvas]:w-0 group-data-[collapsible=offcanvas]:overflow-hidden",
			"w-(--sidebar-width)",
			side === "left" ? "border-r" : "border-l",
			className
		)}
		{...restProps}
	>
		{@render children?.()}
	</aside>
{/if}
```

Note: We're using `<aside>` instead of nested divs, and the width collapse is handled by the component's own classes rather than a separate gap div.

**Step 2: Verify build passes**

Run: `bun run check`

**Step 3: Commit**

```bash
git add src/lib/components/ui/sidebar/sidebar.svelte
git commit -m "refactor: adapt sidebar for CSS Grid layout instead of fixed positioning"
```

---

## Task 3: Update CSS Grid Layout for Sidebar

**Files:**
- Modify: `src/styles/layout.css`

Update the grid layout to handle the sidebar's collapsed state properly.

**Step 1: Update layout.css**

Replace contents of `src/styles/layout.css`:

```css
[data-layout="root"] {
  display: grid;
  grid-template-areas:
    "menu    menu    menu"
    "sidebar main    aside"
    "footer  footer  footer";
  grid-template-rows: var(--layout-menu-h) 1fr var(--layout-footer-h);
  grid-template-columns: auto 1fr auto;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background-color: var(--cm-background);
  color: var(--cm-foreground);
}

[data-layout="menu-bar"] {
  grid-area: menu;
}

[data-layout="sidebar"] {
  grid-area: sidebar;
  overflow-y: auto;
}

[data-layout="main"] {
  grid-area: main;
  overflow: hidden;
}

[data-layout="aside"] {
  grid-area: aside;
  display: none;
}

[data-layout="footer-bar"] {
  grid-area: footer;
}
```

Note: Removed the fixed width from `[data-layout="sidebar"]` since the shadcn sidebar component now controls its own width via `--sidebar-width` CSS variable.

**Step 2: Verify build passes**

Run: `bun run check`

**Step 3: Commit**

```bash
git add src/styles/layout.css
git commit -m "refactor: update grid layout to work with shadcn sidebar"
```

---

## Task 4: Map Sidebar CSS Variables to Tokyo Night Theme

**Files:**
- Modify: `src/app.css`

**Step 1: Add sidebar CSS variable mappings**

Add after the `@theme inline` block (around line 111), before the `@layer base` block:

```css
/* Map sidebar variables to Tokyo Night theme */
:root {
  --sidebar: var(--cm-background-dark);
  --sidebar-foreground: var(--cm-foreground);
  --sidebar-primary: var(--cm-accent);
  --sidebar-primary-foreground: var(--cm-accent-foreground);
  --sidebar-accent: var(--cm-background-highlight);
  --sidebar-accent-foreground: var(--cm-foreground);
  --sidebar-border: var(--cm-gutter-foreground);
  --sidebar-ring: var(--cm-accent);
}
```

Note: These automatically respond to light/dark mode because the `--cm-*` variables are defined per color scheme in `codemirror.css`.

**Step 2: Verify build passes**

Run: `bun run check`

**Step 3: Commit**

```bash
git add src/app.css
git commit -m "feat: map sidebar CSS variables to Tokyo Night theme"
```

---

## Task 5: Update Layout to Use Sidebar.Provider

**Files:**
- Modify: `src/routes/+layout.svelte`

**Step 1: Update layout.svelte**

Replace contents of `src/routes/+layout.svelte`:

```svelte
<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { createDraftsState } from '$lib/stores/drafts.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	let { children } = $props();

	// Create drafts context at layout level
	const draftsState = createDraftsState();

	// Initialize on mount
	onMount(() => {
		draftsState.init();
	});
</script>

<Sidebar.Provider>
	<div data-layout="root">
		{@render children()}
	</div>
</Sidebar.Provider>
```

**Step 2: Verify build passes**

Run: `bun run check`

**Step 3: Commit**

```bash
git add src/routes/+layout.svelte
git commit -m "feat: wrap layout with Sidebar.Provider"
```

---

## Task 6: Create AppSidebar Component

**Files:**
- Create: `src/lib/components/app-sidebar.svelte`

**Step 1: Create the AppSidebar component**

Create `src/lib/components/app-sidebar.svelte`:

```svelte
<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import { Plus } from '@lucide/svelte';

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

<Sidebar.Root collapsible="offcanvas">
	<Sidebar.Header class="p-2">
		<button
			onclick={() => draftsState.newDraft()}
			class="flex w-full items-center justify-center gap-2 rounded-md bg-sidebar-primary px-3 py-2 text-sm font-medium text-sidebar-primary-foreground hover:opacity-90"
		>
			<Plus class="size-4" />
			<span>New Draft</span>
		</button>
	</Sidebar.Header>

	<Sidebar.Content>
		{#if draftsState.isLoading}
			<div class="p-4 text-center text-sm text-sidebar-foreground/70">Loading...</div>
		{:else if draftsState.error}
			<div class="p-4 text-center text-sm text-destructive">{draftsState.error}</div>
		{:else}
			<Sidebar.Group class="p-0">
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each draftsState.drafts as draft (draft.id)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton
									isActive={draftsState.currentDraft?.id === draft.id}
									onclick={() => draftsState.selectDraft(draft.id)}
									class="rounded-none"
								>
									{getDisplayTitle(draft.content)}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/if}
	</Sidebar.Content>
</Sidebar.Root>
```

**Step 2: Verify build passes**

Run: `bun run check`

**Step 3: Commit**

```bash
git add src/lib/components/app-sidebar.svelte
git commit -m "feat: create AppSidebar component using shadcn Sidebar"
```

---

## Task 7: Update Page to Use AppSidebar

**Files:**
- Modify: `src/routes/+page.svelte`
- Delete: `src/lib/components/layout/Sidebar.svelte`
- Modify: `src/lib/components/layout/index.ts`

**Step 1: Update +page.svelte**

Replace contents of `src/routes/+page.svelte`:

```svelte
<script lang="ts">
	import { Editor, createEditorContext } from '$lib/components/editor';
	import { MenuBar, FooterBar, Aside } from '$lib/components/layout';
	import AppSidebar from '$lib/components/app-sidebar.svelte';

	// Create editor context at page level
	const editorState = createEditorContext();
</script>

<MenuBar />
<AppSidebar />
<main data-layout="main">
	<Editor />
</main>
<Aside />
<FooterBar />
```

**Step 2: Update layout/index.ts**

Update `src/lib/components/layout/index.ts`:

```typescript
export { default as MenuBar } from './MenuBar.svelte';
export { default as FooterBar } from './FooterBar.svelte';
export { default as Aside } from './Aside.svelte';
```

**Step 3: Delete old Sidebar.svelte**

```bash
rm src/lib/components/layout/Sidebar.svelte
```

**Step 4: Verify build passes**

Run: `bun run check`

**Step 5: Commit**

```bash
git add src/routes/+page.svelte src/lib/components/layout/index.ts
git rm src/lib/components/layout/Sidebar.svelte
git commit -m "refactor: replace custom Sidebar with AppSidebar"
```

---

## Task 8: Update MenuBar to Use useSidebar

**Files:**
- Modify: `src/lib/components/layout/MenuBar.svelte`
- Delete: `src/lib/stores/ui.svelte.ts`

**Step 1: Update MenuBar.svelte**

Replace contents of `src/lib/components/layout/MenuBar.svelte`:

```svelte
<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, X, PanelLeft } from '@lucide/svelte';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';

	const appWindow = getCurrentWindow();
	const sidebar = useSidebar();

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		await appWindow.toggleMaximize();
	}

	async function close() {
		await appWindow.close();
	}
</script>

<header
	data-layout="menu-bar"
	data-tauri-drag-region
	class="flex h-[var(--layout-menu-h)] items-center justify-between bg-[var(--cm-background-dark)] px-2"
>
	<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
		<button
			onclick={() => sidebar.toggle()}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
			class:text-[var(--cm-accent)]={sidebar.open}
			aria-label="Toggle sidebar"
			aria-pressed={sidebar.open}
		>
			<PanelLeft class="size-3.5" />
		</button>
	</div>
	<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
		<button
			onclick={minimize}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
			aria-label="Minimize"
		>
			<Minus class="size-3.5" />
		</button>
		<button
			onclick={toggleMaximize}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
			aria-label="Maximize"
		>
			<Square class="size-3" />
		</button>
		<button
			onclick={close}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-error)]"
			aria-label="Close"
		>
			<X class="size-3.5" />
		</button>
	</div>
</header>
```

**Step 2: Delete ui.svelte.ts**

```bash
rm src/lib/stores/ui.svelte.ts
```

**Step 3: Verify build passes**

Run: `bun run check`

**Step 4: Commit**

```bash
git add src/lib/components/layout/MenuBar.svelte
git rm src/lib/stores/ui.svelte.ts
git commit -m "refactor: use useSidebar hook instead of custom UIState"
```

---

## Task 9: Final Cleanup and Verification

**Files:**
- Review all modified files

**Step 1: Run full type check**

```bash
bun run check
```

**Step 2: Run Rust check**

```bash
cd src-tauri && cargo check
```

**Step 3: Commit any remaining changes**

```bash
git add -A
git commit -m "chore: cleanup after shadcn sidebar adoption"
```

---

## Task 10: Manual Testing Checklist

> **Note:** This task is for manual verification by the user. The agent should NOT attempt to run the dev server.

**User Action Required:** Run `bun run tauri dev` and verify the following:

### Test Checklist

- [ ] **Sidebar appears on launch** - Left sidebar shows with drafts list
- [ ] **Toggle button works** - PanelLeft icon in menu bar toggles sidebar instantly (no animation)
- [ ] **Keyboard shortcut works** - Cmd+B (Mac) / Ctrl+B (Windows/Linux) toggles sidebar
- [ ] **Theme matches** - Sidebar uses Tokyo Night colors, matches rest of app
- [ ] **Grid layout intact** - MenuBar at top, Sidebar left, Editor center, FooterBar at bottom
- [ ] **New Draft button works** - Creates new draft and adds to list
- [ ] **Draft selection works** - Clicking draft in sidebar loads it in editor
- [ ] **Autosave still works** - Type content, wait, quit and relaunch - content persisted
- [ ] **No animations** - All transitions are instant, no sliding or fading
- [ ] **Sidebar collapse works** - When collapsed, sidebar takes no horizontal space

### If issues are found:

Document the issue and work through debugging. Common issues:
- Sidebar not appearing in grid (check `data-layout` attributes)
- Theme mismatch (check CSS variable mapping in Task 4)
- Toggle not working (check useSidebar context)
- Grid layout broken (check layout.css and component order)

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 0 | Add type helpers | `utils.ts` |
| 1 | Add duration constants | `constants.ts`, `sidebar.svelte`, `sidebar-provider.svelte`, `sidebar-group-label.svelte`, `sheet/constants.ts`, `sheet-content.svelte` |
| 2 | Adapt sidebar for grid | `sidebar.svelte` |
| 3 | Update grid layout CSS | `layout.css` |
| 4 | Map CSS variables | `app.css` |
| 5 | Add Sidebar.Provider | `+layout.svelte` |
| 6 | Create AppSidebar | `app-sidebar.svelte` |
| 7 | Update page | `+page.svelte`, remove `Sidebar.svelte` |
| 8 | Update MenuBar | `MenuBar.svelte`, remove `ui.svelte.ts` |
| 9 | Cleanup | All files |
| 10 | Manual testing | User verification |

## Future Considerations

- **Right sidebar (Actions panel)**: Can add another `Sidebar.Root` with `side="right"` - will need to extend to support multiple sidebars like honeydew's approach
- **Icon collapse mode**: Change `collapsible="offcanvas"` to `collapsible="icon"` for a collapsed-to-icons mode
- **Mobile support**: The Sheet component handles mobile automatically
