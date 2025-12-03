# Route Refactor Design

**Date:** 2025-01-03

## Problem

The current frontend uses a React-ish SPA pattern: one route (`/`) handles everything via client-side state. This works for Tauri but has drawbacks:

- URLs don't reflect current view (no `/drafts/123`)
- Browser back/forward doesn't work between drafts
- Not shareable/bookmarkable (matters for future web frontend)
- Single `+page.svelte` mixes concerns

## Goals

1. **Shareable URLs** - `/drafts/123` works when shared or bookmarked
2. **Navigation UX** - Browser back/forward works between drafts
3. **Code organization** - Route-based components with single responsibilities

## Route Structure

```
src/routes/
├── +layout.svelte           # Minimal: global styles + {@render children()}
├── +layout.ts               # ssr = false
├── +page.ts                 # redirect(307, '/drafts')
│
├── (editor)/                # Layout group for editor shell
│   ├── +layout.svelte       # Editor shell: Sidebar + WinBar + StatusLine
│   ├── +layout.ts           # load(): returns { drafts: Draft[] }
│   ├── WinBar.svelte        # Co-located
│   ├── StatusLine.svelte    # Co-located
│   │
│   └── drafts/
│       ├── +page.ts         # redirect to /drafts/[most-recent] or /drafts/new
│       │
│       ├── new/
│       │   └── +page.svelte # Empty editor, creates on first save, replaceState
│       │
│       └── [id]/
│           ├── +page.svelte # Editor with loaded draft
│           └── +page.ts     # load(): returns { draft: Draft }
│
└── settings/                # Future: outside (editor), different/no layout
    └── ...
```

## URL Behavior

| URL | Behavior |
|-----|----------|
| `/` | Redirect to `/drafts` |
| `/drafts` | Redirect to `/drafts/[most-recent]` or `/drafts/new` if none |
| `/drafts/new` | Empty editor, creates draft on first save, `replaceState` to `/drafts/[id]` |
| `/drafts/[id]` | Editor with loaded draft |

Redirects happen in load functions (instant, no flash).

## Data Flow

### Draft List
- Loaded in `(editor)/+layout.ts`
- Available to sidebar and child routes via `data.drafts`

### Current Draft
- Loaded in `(editor)/drafts/[id]/+page.ts`
- Passed to Editor component as props

### Mutations
- Call functions from `$lib/api` directly
- Then `invalidateAll()` to refresh data
- Then `goto()` to navigate (e.g., after delete)

## State Migration

The `DraftsState` class currently handles too many concerns. Split as follows:

| Was | Becomes |
|-----|---------|
| `DraftsState.drafts` | `(editor)/+layout.ts` load function |
| `DraftsState.currentDraft` | Route params + `drafts/[id]/+page.ts` load |
| `DraftsState.selectDraft()` | `<a href="/drafts/{id}">` links |
| `DraftsState.newDraft()` | `goto('/drafts/new')` |
| `DraftsState.removeDraft()` | `deleteDraft()` + `invalidateAll()` + `goto()` |
| `DraftsState` autosave | Editor component owns debounced save |

## Component Changes

### Editor.svelte

Changes from context-based to props-based:

```svelte
// Before
const draftsState = getDraftsState();
// Uses draftsState.currentDraft, draftsState.updateContent()

// After
interface Props {
  initialContent: string;
  draftId?: number;  // undefined for /drafts/new
  onContentChange?: (content: string) => void;
}
let { initialContent, draftId, onContentChange }: Props = $props();
```

The `$effect` watching `currentDraft` goes away - route navigation handles switching drafts by mounting a fresh page with different props.

### WinBar.svelte

Uses `$app/state` for route awareness instead of `DraftsState`:

```ts
import { page } from '$app/state';
import { goto, invalidateAll } from '$app/navigation';
import { deleteDraft } from '$lib/api';

const currentDraftId = $derived(page.params.id ? Number(page.params.id) : null);

// New draft: just navigate
function handleNew() {
  goto('/drafts/new');
}

// Delete: API call + invalidate + navigate
async function handleDelete() {
  if (!currentDraftId) return;
  await deleteDraft(currentDraftId);
  await invalidateAll();
  goto('/drafts');  // Will redirect to next draft or /drafts/new
}
```

### Autosave

Moves into Editor component or page components:
- Use `beforeNavigate` from `$app/navigation` to flush pending saves
- Debounce logic stays similar, just lives closer to where it's used

## What Gets Deleted

- `DraftsState` class (keep only `Draft` model class for derived properties)
- Context machinery (`createDraftsState`, `getDraftsState`, `setDraftsState`)
- Current root `+page.svelte` content (moves to route-specific files)

## What Stays

- `Draft` class in `$lib/stores/drafts.svelte.ts` - useful reactive model with derived `title`, `previewLines`, `formattedModifiedAt`
- `$lib/api/*` - already clean standalone functions
- Editor component in `$lib/components/editor/` - gains autosave but otherwise stays

## Future Extensibility

- Add `/templates`, `/snippets` inside `(editor)/` to get sidebar shell automatically
- Add `/settings` outside `(editor)/` for full-screen, no-sidebar layout
- Web frontend gets same routes, just different adapter

## Key Behaviors

1. **Fast redirects** - All redirects in load functions, no client-side flash
2. **Autosave flush** - `beforeNavigate` ensures pending saves complete before navigation
3. **Create on save** - `/drafts/new` only persists on first keystroke/save
4. **URL reflects state** - Current draft ID always in URL (except `/drafts/new`)
