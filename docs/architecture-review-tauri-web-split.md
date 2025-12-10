# Architecture Review: Tauri Desktop vs. Web Split

This document reviews the current workspace architecture for the Dashtext application, which targets both a Tauri desktop app and a Cloudflare Workers web app using a Bun monorepo structure.

## Table of Contents

- [Current State Summary](#current-state-summary)
- [What's Working Well](#whats-working-well)
- [Current Duplication (Drift Risk)](#current-duplication-drift-risk)
- [Recommendation: Keep Separate, But Reduce Duplication](#recommendation-keep-separate-but-reduce-duplication)
- [Proposed Solutions](#proposed-solutions)
  - [1. Composable Layout Components](#1-composable-layout-components)
  - [2. Platform Capability Injection](#2-platform-capability-injection)
  - [3. Drift Prevention Strategy](#3-drift-prevention-strategy)
- [Migration Path](#migration-path)
- [Proposed Architecture Diagram](#proposed-architecture-diagram)
- [Summary](#summary)
- [Svelte 5 Compliance Review](#svelte-5-compliance-review)
- [Implementation Checklist](#implementation-checklist)
- [Additional Patterns to Review](#additional-patterns-to-review)

---

## Current State Summary

The workspace has three packages:

| Package | Purpose | Storage | Adapter |
|---------|---------|---------|---------|
| `@dashtext/lib` | Shared UI components, stores, API types | N/A | N/A |
| `@dashtext/web` | Cloudflare Workers web app | IndexedDB | `adapter-cloudflare` |
| `@dashtext/desktop` | Tauri native desktop app | SQLite (Drizzle) | `adapter-static` |

### Package Structure

```
dashtext/
├── package.json              # Root workspace config
├── tsconfig.json             # Shared TypeScript config
├── bun.lock                  # Monorepo lockfile
├── packages/
│   ├── lib/                  # Shared components & utilities
│   │   ├── src/
│   │   │   ├── components/   # UI components (editor, shadcn-svelte)
│   │   │   ├── stores/       # DraftsState, Draft classes
│   │   │   └── api/          # DraftAPI interface
│   │   └── package.json
│   ├── web/                  # Cloudflare Workers app
│   │   ├── src/
│   │   │   ├── lib/api/      # IndexedDB backend implementation
│   │   │   └── routes/       # SvelteKit routes
│   │   ├── wrangler.jsonc
│   │   └── package.json
│   └── desktop/              # Tauri desktop app
│       ├── src/
│       │   ├── lib/
│       │   │   ├── api/      # SQLite backend implementation
│       │   │   ├── db/       # Drizzle schema & migrations
│       │   │   ├── components/  # Updater, capture (desktop-only)
│       │   │   └── stores/   # Updater state
│       │   └── routes/       # SvelteKit routes
│       ├── src-tauri/        # Rust backend
│       └── package.json
└── .github/workflows/        # Build & release pipelines
```

---

## What's Working Well

### 1. API Abstraction Pattern

The `DraftAPI` interface in lib with platform-specific implementations is well-designed:

```
@dashtext/lib/api/types.ts (interface definition)
       ↓
@dashtext/desktop/src/lib/api/backend.ts (SQLite via Drizzle)
@dashtext/web/src/lib/api/backend.ts (IndexedDB via idb)
```

Both backends satisfy the same contract, allowing the shared `DraftsState` class to work identically on both platforms.

```typescript
// packages/lib/src/api/types.ts
export interface DraftAPI {
  list(): Promise<DraftData[]>;
  create(): Promise<DraftData>;
  get(id: number): Promise<DraftData | null>;
  save(id: number, content: string): Promise<DraftData>;
  delete(id: number): Promise<void>;
}
```

### 2. Shared UI Components

The following are properly shared via `@dashtext/lib`:

- **Editor**: CodeMirror 6 with Vim mode (`@replit/codemirror-vim`)
- **UI Components**: shadcn-svelte pattern (button, dialog, sidebar, tooltip, etc.)
- **AppBar Primitives**: Composable header/footer components
- **State Management**: `DraftsState` and `Draft` classes using Svelte 5 runes

### 3. Workspace Dependency Management

The Bun workspace catalog keeps dependency versions synchronized across packages:

```json
// root package.json
{
  "workspaces": ["packages/*"],
  "catalog": {
    "codemirror": "^6.0.1",
    "svelte": "^5.33.0",
    "@sveltejs/kit": "^2.21.0"
    // ...
  }
}
```

---

## Current Duplication (Drift Risk)

Several files are duplicated between `web` and `desktop` with minor differences:

| File | Similarity | Differences |
|------|------------|-------------|
| `StatusLine.svelte` | ~95% | Desktop adds `<VersionIndicator />` |
| `WinBar.svelte` | ~70% | Desktop adds quick capture button, `<WindowControls />`, Tauri drag region attributes |
| `(editor)/+layout.svelte` | ~80% | Desktop adds updater state initialization, Tauri focus listener vs. browser visibility API |

### Detailed Comparison

#### StatusLine.svelte

**Web version:**
```svelte
<AppBar.Section>
  <VimModeIndicator />
</AppBar.Section>
```

**Desktop version:**
```svelte
<AppBar.Section>
  <VimModeIndicator />
  <VersionIndicator />  <!-- Desktop-only -->
</AppBar.Section>
```

#### WinBar.svelte

**Shared functionality (~70%):**
- Sidebar toggle button
- New draft button
- Delete draft button

**Desktop-only additions:**
- Quick capture button (`<Zap />` icon)
- `<WindowControls />` component (minimize, maximize, close)
- `data-tauri-drag-region` attribute
- `-webkit-app-region: no-drag` styles on interactive elements

#### (editor)/+layout.svelte

**Shared functionality (~80%):**
- Editor context creation
- DraftsState initialization with API adapter
- Sidebar rendering with draft list
- Main content area structure

**Platform-specific:**

| Aspect | Web | Desktop |
|--------|-----|---------|
| Focus detection | `document.visibilitychange` | `getCurrentWindow().onFocusChanged()` |
| Updater | N/A | `createUpdaterState()` + `<UpdateDialog />` |
| Tooltip provider | Wraps WinBar | Not present (handled differently) |

---

## Recommendation: Keep Separate, But Reduce Duplication

### Do NOT Combine Into One Package

The targets are fundamentally different and combining them would increase complexity:

| Aspect | Desktop | Web |
|--------|---------|-----|
| **Runtime** | Native (Tauri/Rust) | Browser (Cloudflare Workers) |
| **SvelteKit Adapter** | `adapter-static` (SPA) | `adapter-cloudflare` (Workers) |
| **Storage** | SQLite via Tauri SQL plugin | IndexedDB (future: D1) |
| **Distribution** | .deb, .AppImage, .rpm | URL deployment |
| **Updates** | Self-updating via GitHub releases | Browser refresh |
| **Window Management** | Custom titlebar, native controls | Browser chrome |
| **Additional Features** | Quick capture, updater | N/A |

### Extract More Shared Structure

The `DraftAPI` abstraction is a good model—apply the same pattern to layout components and platform behaviors.

---

## Proposed Solutions

### 1. Composable Layout Components

Extract shared layout structure to `@dashtext/lib` with slot-based composition for platform-specific additions.

#### New Directory Structure

```
packages/lib/src/components/editor-layout/
├── EditorLayout.svelte      # Common structure with slots
├── DraftsSidebar.svelte     # Extract sidebar rendering
├── BaseStatusLine.svelte    # Status bar with extension slot
├── BaseWinBar.svelte        # Toolbar with extension slots
└── index.ts
```

#### EditorLayout.svelte

```svelte
<!-- packages/lib/src/components/editor-layout/EditorLayout.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { createEditorContext } from '$lib/editor';
  import { createDraftsState, type DraftsAPI } from '$lib/stores';
  import { getPlatform } from '$lib/platform';
  import * as Sidebar from '$lib/sidebar';
  import DraftsSidebar from './DraftsSidebar.svelte';
  import BaseStatusLine from './BaseStatusLine.svelte';
  import BaseWinBar from './BaseWinBar.svelte';

  type Props = {
    drafts: DraftData[];
    draftsAPI: DraftsAPI;
    refreshDrafts: () => Promise<DraftData[]>;
    headerExtra?: Snippet;      // Platform-specific toolbar buttons
    statusExtra?: Snippet;      // Platform-specific status indicators
    afterLayout?: Snippet;      // Modals, dialogs (e.g., UpdateDialog)
    children: Snippet;
  };

  let {
    drafts,
    draftsAPI,
    refreshDrafts,
    headerExtra,
    statusExtra,
    afterLayout,
    children
  }: Props = $props();

  const platform = getPlatform();

  createEditorContext();
  const draftsState = createDraftsState(() => drafts, draftsAPI);

  // Platform-agnostic focus handling
  $effect(() => {
    const unsubscribe = platform.onFocusChange(() => {
      refreshDrafts().then((fresh) => {
        draftsState.drafts = fresh;
      });
    });
    return unsubscribe;
  });
</script>

<Sidebar.Provider>
  <div data-layout="root">
    <BaseWinBar>
      {#if headerExtra}{@render headerExtra()}{/if}
    </BaseWinBar>

    <Sidebar.Root collapsible="offcanvas">
      <DraftsSidebar {draftsState} />
    </Sidebar.Root>

    <main data-layout="main">
      {@render children()}
    </main>

    <aside data-layout="aside">
      <!-- Future: aside content -->
    </aside>

    <BaseStatusLine>
      {#if statusExtra}{@render statusExtra()}{/if}
    </BaseStatusLine>
  </div>

  {#if afterLayout}{@render afterLayout()}{/if}
</Sidebar.Provider>
```

#### Platform Layouts Become Thin Wrappers

**Desktop:**
```svelte
<!-- packages/desktop/src/routes/(editor)/+layout.svelte -->
<script lang="ts">
  import { EditorLayout } from '@dashtext/lib/editor-layout';
  import { listDrafts, createDraft, saveDraft } from '$lib/api';
  import { type DraftsAPI } from '@dashtext/lib/stores';
  import { replaceState } from '$app/navigation';
  import { createUpdaterState } from '$lib/stores/updater.svelte';
  import { UpdateDialog, VersionIndicator } from '$lib/components/updater';
  import { openQuickCapture } from '$lib/components/capture';
  import WindowControls from './WindowControls.svelte';
  import { Button } from '@dashtext/lib/button';
  import { Zap } from '@lucide/svelte';

  let { data, children } = $props();

  const draftsAPI: DraftsAPI = {
    createDraft: async () => createDraft(),
    saveDraft: async (id, content) => saveDraft(id, content),
    replaceUrl: (url) => replaceState(url, {})
  };

  const updater = createUpdaterState();
  $effect(() => updater.init());
</script>

<EditorLayout
  drafts={data.drafts}
  {draftsAPI}
  refreshDrafts={listDrafts}
>
  {#snippet headerExtra()}
    <Button variant="toolbar" size="icon-sm" onclick={() => openQuickCapture()}>
      <Zap class="size-3.5" />
    </Button>
    <WindowControls />
  {/snippet}

  {#snippet statusExtra()}
    <VersionIndicator />
  {/snippet}

  {#snippet afterLayout()}
    <UpdateDialog />
  {/snippet}

  {@render children()}
</EditorLayout>
```

**Web:**
```svelte
<!-- packages/web/src/routes/(editor)/+layout.svelte -->
<script lang="ts">
  import { EditorLayout } from '@dashtext/lib/editor-layout';
  import { listDrafts, createDraft, saveDraft } from '$lib/api';
  import { type DraftsAPI } from '@dashtext/lib/stores';
  import { replaceState } from '$app/navigation';

  let { data, children } = $props();

  const draftsAPI: DraftsAPI = {
    createDraft: async () => createDraft(),
    saveDraft: async (id, content) => saveDraft(id, content),
    replaceUrl: (url) => replaceState(url, {})
  };
</script>

<EditorLayout
  drafts={data.drafts}
  {draftsAPI}
  refreshDrafts={listDrafts}
>
  {@render children()}
</EditorLayout>
```

---

### 2. Platform Capability Injection

Abstract platform-specific behaviors behind a common interface, enabling shared components to adapt without importing platform-specific code.

#### Define the Capability Interface

```typescript
// packages/lib/src/platform/types.ts
export interface PlatformCapabilities {
  /** Platform identifier for conditional rendering */
  platform: 'desktop' | 'web';

  /** Subscribe to app focus changes, returns unsubscribe function */
  onFocusChange(callback: () => void): () => void;

  /** Window control capabilities (null if not supported) */
  window: {
    minimize(): Promise<void>;
    maximize(): Promise<void>;
    close(): Promise<void>;
    startDrag(): Promise<void>;
  } | null;

  /** Updater capabilities (null if not supported) */
  updater: {
    checkForUpdate(): Promise<UpdateInfo | null>;
    downloadAndInstall(info: UpdateInfo): Promise<void>;
  } | null;

  /** Quick capture (null if not supported) */
  quickCapture: {
    open(): Promise<void>;
  } | null;
}

export interface UpdateInfo {
  version: string;
  notes: string;
  downloadUrl: string;
}
```

#### Create Context Helpers

> **Note**: This uses Svelte 5.40.0+'s `createContext()` for type-safe, key-free context management, consistent with the existing `getDraftsState`/`setDraftsState` and `getEditorContext`/`setEditorContext` patterns in the codebase.

```typescript
// packages/lib/src/platform/context.ts
import { createContext } from 'svelte';
import type { PlatformCapabilities } from './types';

// Type-safe context using Svelte 5.40.0+ createContext()
// Returns [getter, setter] tuple - no key needed
export const [getPlatform, setPlatformContext] = createContext<PlatformCapabilities>();
```

```typescript
// packages/lib/src/platform/index.ts
export { getPlatform, setPlatformContext } from './context';
export type { PlatformCapabilities, UpdateInfo } from './types';
```

#### Implement for Desktop (Tauri)

```typescript
// packages/desktop/src/lib/platform.ts
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { PlatformCapabilities } from '@dashtext/lib/platform';
import { checkForUpdate, downloadAndInstall } from './updater';

export const desktopPlatform: PlatformCapabilities = {
  platform: 'desktop',

  onFocusChange(callback) {
    let unlisten: (() => void) | null = null;

    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused) callback();
      })
      .then((fn) => { unlisten = fn; });

    return () => unlisten?.();
  },

  window: {
    minimize: () => getCurrentWindow().minimize(),
    maximize: () => getCurrentWindow().toggleMaximize(),
    close: () => getCurrentWindow().close(),
    startDrag: () => getCurrentWindow().startDragging(),
  },

  updater: {
    checkForUpdate,
    downloadAndInstall,
  },

  quickCapture: {
    open: () => import('./components/capture').then(m => m.openQuickCapture()),
  },
};
```

#### Implement for Web

```typescript
// packages/web/src/lib/platform.ts
import type { PlatformCapabilities } from '@dashtext/lib/platform';

export const webPlatform: PlatformCapabilities = {
  platform: 'web',

  onFocusChange(callback) {
    const handler = () => {
      if (document.visibilityState === 'visible') callback();
    };
    document.addEventListener('visibilitychange', handler);
    return () => document.removeEventListener('visibilitychange', handler);
  },

  // Not available on web
  window: null,
  updater: null,
  quickCapture: null,
};
```

#### Initialize at App Root

```svelte
<!-- packages/desktop/src/routes/+layout.svelte -->
<script lang="ts">
  import { setPlatformContext } from '@dashtext/lib/platform';
  import { desktopPlatform } from '$lib/platform';

  let { children } = $props();

  setPlatformContext(desktopPlatform);
</script>

{@render children()}
```

```svelte
<!-- packages/web/src/routes/+layout.svelte -->
<script lang="ts">
  import { setPlatformContext } from '@dashtext/lib/platform';
  import { webPlatform } from '$lib/platform';

  let { children } = $props();

  setPlatformContext(webPlatform);
</script>

{@render children()}
```

#### Consume in Shared Components

```svelte
<!-- packages/lib/src/components/editor-layout/BaseWinBar.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { getPlatform } from '$lib/platform';
  import * as AppBar from '$lib/appbar';
  import { Button } from '$lib/button';
  import * as Tooltip from '$lib/tooltip';
  import { useSidebar } from '$lib/sidebar';
  import { getDraftsState } from '$lib/stores';
  import { goto } from '$app/navigation';
  import { PanelLeft, Plus, Trash2, Zap, Minus, Square, X } from '@lucide/svelte';

  type Props = {
    children?: Snippet;  // Platform-specific extra content
  };

  let { children }: Props = $props();

  const platform = getPlatform();
  const windowControls = platform.window;
  const quickCapture = platform.quickCapture;
  const sidebar = useSidebar();
  const draftsState = getDraftsState();

  const currentDraftId = $derived(draftsState.currentDraft?.id ?? null);

  function handleNew() {
    goto('/drafts/new');
  }

  async function handleDelete() {
    if (!currentDraftId) return;
    // Deletion logic...
  }
</script>

<AppBar.Root
  as="header"
  data-layout="menu-bar"
  data-tauri-drag-region={platform.platform === 'desktop' ? true : undefined}
  class="gap-1 bg-[var(--cm-background-dark)] p-1"
>
  <AppBar.Section style={platform.platform === 'desktop' ? '-webkit-app-region: no-drag;' : ''}>
    <!-- Shared buttons -->
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="toolbar"
            size="icon-sm"
            onclick={() => sidebar.toggle()}
            aria-label="Toggle sidebar"
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
          <Button {...props} variant="toolbar" size="icon-sm" onclick={handleNew}>
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
          >
            <Trash2 class="size-3.5" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content side="bottom">Delete draft</Tooltip.Content>
    </Tooltip.Root>

    <!-- Quick capture: only renders if capability exists -->
    {#if quickCapture}
      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button
              {...props}
              variant="toolbar"
              size="icon-sm"
              onclick={() => quickCapture.open()}
            >
              <Zap class="size-3.5" />
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content side="bottom">Quick capture</Tooltip.Content>
      </Tooltip.Root>
    {/if}

    <!-- Platform-specific extra content via slot -->
    {#if children}{@render children()}{/if}
  </AppBar.Section>

  <!-- Window controls: only on desktop -->
  {#if windowControls}
    <AppBar.Section style="-webkit-app-region: no-drag;">
      <Button variant="toolbar" size="icon-sm" onclick={() => windowControls.minimize()}>
        <Minus class="size-3.5" />
      </Button>
      <Button variant="toolbar" size="icon-sm" onclick={() => windowControls.maximize()}>
        <Square class="size-3.5" />
      </Button>
      <Button variant="toolbar" size="icon-sm" onclick={() => windowControls.close()}>
        <X class="size-3.5" />
      </Button>
    </AppBar.Section>
  {/if}
</AppBar.Root>
```

#### Benefits of Capability Injection

| Benefit | Description |
|---------|-------------|
| **No platform imports in shared code** | Lib never imports `@tauri-apps/*` or browser-specific APIs directly |
| **Type-safe capability checking** | `if (windowControls)` narrows the type, no runtime errors |
| **Easy to extend** | Add new capabilities (e.g., file system access, notifications) without changing shared components |
| **Testable** | Inject mock capabilities for testing shared components |
| **Clear boundaries** | Platform-specific code stays in platform packages |

#### Testing with Mock Platform

```typescript
// For component tests
export const mockPlatform: PlatformCapabilities = {
  platform: 'web',
  onFocusChange: () => () => {},
  window: null,
  updater: null,
  quickCapture: null,
};

// Test desktop-specific behavior
export const mockDesktopPlatform: PlatformCapabilities = {
  platform: 'desktop',
  onFocusChange: (cb) => {
    // Can trigger manually in tests
    return () => {};
  },
  window: {
    minimize: vi.fn(),
    maximize: vi.fn(),
    close: vi.fn(),
    startDrag: vi.fn(),
  },
  updater: {
    checkForUpdate: vi.fn(),
    downloadAndInstall: vi.fn(),
  },
  quickCapture: {
    open: vi.fn(),
  },
};
```

---

### 3. Drift Prevention Strategy

Add automated checks to detect when shared patterns diverge.

#### Option A: Diff Check in CI

```yaml
# .github/workflows/check-drift.yml
name: Check Platform Drift

on: [push, pull_request]

jobs:
  check-drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check for layout drift
        run: |
          # Compare files that should stay in sync
          files_to_check=(
            "src/routes/(editor)/drafts/+page.svelte"
            "src/routes/(editor)/drafts/[[id]]/+page.svelte"
          )

          for file in "${files_to_check[@]}"; do
            web="packages/web/$file"
            desktop="packages/desktop/$file"

            if [ -f "$web" ] && [ -f "$desktop" ]; then
              if ! diff -q "$web" "$desktop" > /dev/null 2>&1; then
                echo "⚠️  Drift detected in $file"
                diff -u "$web" "$desktop" || true
              fi
            fi
          done
```

#### Option B: Custom Drift Detection Script

```typescript
// scripts/check-drift.ts
import { readFileSync, existsSync } from 'fs';
import { glob } from 'glob';

interface DriftReport {
  file: string;
  webOnly: string[];
  desktopOnly: string[];
  shared: string[];
}

// Files that SHOULD be identical
const MUST_MATCH = [
  'src/routes/(editor)/drafts/+page.svelte',
  'src/routes/(editor)/drafts/+page.ts',
  'src/routes/(editor)/drafts/[[id]]/+page.svelte',
];

// Files that are allowed to differ
const ALLOWED_DIFFERENT = [
  'src/routes/(editor)/+layout.svelte',
  'src/routes/(editor)/StatusLine.svelte',
  'src/routes/(editor)/WinBar.svelte',
];

for (const file of MUST_MATCH) {
  const webPath = `packages/web/${file}`;
  const desktopPath = `packages/desktop/${file}`;

  if (existsSync(webPath) && existsSync(desktopPath)) {
    const webContent = readFileSync(webPath, 'utf-8');
    const desktopContent = readFileSync(desktopPath, 'utf-8');

    if (webContent !== desktopContent) {
      console.error(`❌ DRIFT: ${file} differs between web and desktop`);
      process.exitCode = 1;
    }
  }
}
```

#### Option C: Shared Route Symlinks (Not Recommended)

Symlinks can cause issues with some bundlers and add complexity. Prefer extracting to lib instead.

---

## Migration Path

### Phase 1: Extract DraftsSidebar (Low Risk)

The sidebar rendering is 100% duplicated. Extract to lib immediately.

1. Create `packages/lib/src/components/editor-layout/DraftsSidebar.svelte`
2. Move sidebar rendering logic from both layouts
3. Import and use in both platform layouts
4. Verify both platforms work correctly

### Phase 2: Create Platform Abstraction

1. Create `packages/lib/src/platform/` with types and context
2. Implement `desktopPlatform` in desktop package
3. Implement `webPlatform` in web package
4. Initialize platform context in root layouts
5. Verify platform detection works

### Phase 3: Composable StatusLine and WinBar

1. Create `BaseStatusLine.svelte` with slot for extras
2. Create `BaseWinBar.svelte` with platform capability detection
3. Migrate desktop to use base components + extras
4. Migrate web to use base components
5. Remove duplicated platform-specific code

### Phase 4: Full EditorLayout Extraction

1. Create `EditorLayout.svelte` that composes all pieces
2. Migrate desktop layout to thin wrapper
3. Migrate web layout to thin wrapper
4. Add drift detection CI check
5. Document the pattern for future development

---

## Proposed Architecture Diagram

### Current Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         @dashtext/lib                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   editor/   │  │     ui/     │  │   stores/   │             │
│  │ (CodeMirror)│  │ (shadcn-sv) │  │(DraftsState)│             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│  ┌─────────────┐  ┌─────────────┐                              │
│  │   appbar/   │  │    api/     │                              │
│  │ (primitives)│  │ (DraftAPI)  │                              │
│  └─────────────┘  └─────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
                    ▲                           ▲
                    │                           │
       ┌────────────┴────────────┐ ┌────────────┴────────────┐
       │     @dashtext/web       │ │    @dashtext/desktop    │
       │  ┌──────────────────┐   │ │  ┌──────────────────┐   │
       │  │  routes/(editor) │   │ │  │  routes/(editor) │   │
       │  │  - +layout.svelte│   │ │  │  - +layout.svelte│   │
       │  │  - StatusLine    │   │ │  │  - StatusLine    │   │
       │  │  - WinBar        │ ◄─┼─┼──│  - WinBar        │   │
       │  └──────────────────┘   │ │  │  - WindowControls│   │
       │  ┌──────────────────┐   │ │  └──────────────────┘   │
       │  │     lib/api/     │   │ │  ┌──────────────────┐   │
       │  │   (IndexedDB)    │   │ │  │     lib/api/     │   │
       │  └──────────────────┘   │ │  │    (SQLite)      │   │
       └─────────────────────────┘ │  └──────────────────┘   │
                                   │  ┌──────────────────┐   │
                    ▲──── DRIFT ───│  │  lib/components/ │   │
                                   │  │ (updater,capture)│   │
                                   │  └──────────────────┘   │
                                   │  ┌──────────────────┐   │
                                   │  │    src-tauri/    │   │
                                   │  │   (Rust backend) │   │
                                   │  └──────────────────┘   │
                                   └─────────────────────────┘
```

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         @dashtext/lib                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   editor/   │  │     ui/     │  │   stores/   │             │
│  │ (CodeMirror)│  │ (shadcn-sv) │  │(DraftsState)│             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   appbar/   │  │    api/     │  │  platform/  │  ◄── NEW    │
│  │ (primitives)│  │ (DraftAPI)  │  │ (capability │             │
│  └─────────────┘  └─────────────┘  │  interface) │             │
│                                    └─────────────┘             │
│  ┌─────────────────────────────────────────────────┐           │
│  │              editor-layout/                      │  ◄── NEW │
│  │  EditorLayout, DraftsSidebar, BaseWinBar,       │           │
│  │  BaseStatusLine (slot-based composition)        │           │
│  └─────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
                    ▲                           ▲
                    │                           │
       ┌────────────┴────────────┐ ┌────────────┴────────────┐
       │     @dashtext/web       │ │    @dashtext/desktop    │
       │  ┌──────────────────┐   │ │  ┌──────────────────┐   │
       │  │  lib/platform.ts │   │ │  │  lib/platform.ts │   │
       │  │  (webPlatform)   │   │ │  │ (desktopPlatform)│   │
       │  └──────────────────┘   │ │  └──────────────────┘   │
       │  ┌──────────────────┐   │ │  ┌──────────────────┐   │
       │  │  routes/(editor) │   │ │  │  routes/(editor) │   │
       │  │  +layout.svelte  │   │ │  │  +layout.svelte  │   │
       │  │  (thin wrapper)  │   │ │  │  (thin wrapper   │   │
       │  └──────────────────┘   │ │  │  + desktop slots)│   │
       │  ┌──────────────────┐   │ │  └──────────────────┘   │
       │  │     lib/api/     │   │ │  ┌──────────────────┐   │
       │  │   (IndexedDB)    │   │ │  │     lib/api/     │   │
       │  └──────────────────┘   │ │  │    (SQLite)      │   │
       └─────────────────────────┘ │  └──────────────────┘   │
                                   │  ┌──────────────────┐   │
                                   │  │  lib/components/ │   │
                                   │  │ (updater,capture)│   │
                                   │  └──────────────────┘   │
                                   │  ┌──────────────────┐   │
                                   │  │    src-tauri/    │   │
                                   │  │   (Rust backend) │   │
                                   │  └──────────────────┘   │
                                   └─────────────────────────┘
```

---

## Summary

| Question | Answer |
|----------|--------|
| **Should they be kept separate?** | **Yes** - Different runtimes, adapters, storage backends, and platform capabilities warrant separation |
| **How to prevent drift?** | Extract shared layout structure to lib with slot-based composition; add CI drift detection |
| **Combine to one package?** | **No** - Would increase complexity without benefit; the differences are fundamental |

### Key Takeaways

1. The **workspace structure is correct** - three packages is the right approach
2. The **`DraftAPI` abstraction is a good model** - extend this pattern to platform capabilities
3. **Extract more to `@dashtext/lib`** - layout components, sidebar rendering, platform abstraction
4. **Use slot-based composition** - platform-specific code injects into shared components via Svelte snippets
5. **Add drift detection** - automated CI checks prevent silent divergence

The goal is to make platform packages as thin as possible: they provide platform-specific implementations and inject them into shared components, rather than duplicating shared logic.

---

## Svelte 5 Compliance Review

This document was reviewed against Svelte 5 documentation on 2024-12-09. All proposed code samples have been validated with `svelte-autofixer`.

### Validated Patterns

| Pattern | Status | Notes |
|---------|--------|-------|
| `$props()` with destructuring | ✅ | Matches codebase pattern |
| `$derived()` / `$derived.by()` | ✅ | Used correctly for computed values |
| `$effect()` with cleanup | ✅ | Returns unsubscribe functions correctly |
| `Snippet` type from `'svelte'` | ✅ | Correct typing for optional/required snippets |
| `{#snippet}` / `{@render}` | ✅ | Correct Svelte 5 syntax |
| `createContext()` tuple pattern | ✅ | Updated to match codebase (Svelte 5.40.0+) |
| Event handlers (`onclick`) | ✅ | Uses Svelte 5 syntax, not legacy `on:click` |

### Codebase Consistency Notes

1. **Context Pattern**: The codebase uses `createContext()` from Svelte 5.40.0+ for drafts and editor contexts. The sidebar uses the older `setContext`/`getContext` with `Symbol.for()` pattern (from shadcn-svelte). The Platform Capability Injection section has been updated to use `createContext()` for consistency.

2. **Naming Conventions**: The codebase mixes `get*` (drafts, editor) and `use*` (sidebar) naming. This document uses `getPlatform()` for the main context accessor to match the existing pattern.

3. **API Index Duplication**: `packages/{desktop,web}/src/lib/api/index.ts` files are nearly identical and could be extracted to the shared lib as a future improvement.

---

## Implementation Checklist

### Phase 1: Extract DraftsSidebar (Low Risk)

- [ ] Create `packages/lib/src/components/editor-layout/` directory
- [ ] Create `DraftsSidebar.svelte` extracting sidebar content from both layouts
- [ ] Export from `packages/lib/src/components/editor-layout/index.ts`
- [ ] Update desktop `(editor)/+layout.svelte` to use `DraftsSidebar`
- [ ] Update web `(editor)/+layout.svelte` to use `DraftsSidebar`
- [ ] Verify both platforms render correctly
- [ ] Run `bun run check` to verify types

### Phase 2: Platform Abstraction

- [ ] Create `packages/lib/src/platform/types.ts` with `PlatformCapabilities` interface
- [ ] Create `packages/lib/src/platform/context.ts` with `createContext()` pattern
- [ ] Create `packages/lib/src/platform/index.ts` barrel export
- [ ] Create `packages/desktop/src/lib/platform.ts` with `desktopPlatform` implementation
- [ ] Create `packages/web/src/lib/platform.ts` with `webPlatform` implementation
- [ ] Update `packages/desktop/src/routes/+layout.svelte` to call `setPlatformContext()`
- [ ] Update `packages/web/src/routes/+layout.svelte` to call `setPlatformContext()`
- [ ] Run `bun run check` and `bun run tauri dev` to verify

### Phase 3: Composable StatusLine and WinBar

- [ ] Create `packages/lib/src/components/editor-layout/BaseStatusLine.svelte`
- [ ] Create `packages/lib/src/components/editor-layout/BaseWinBar.svelte`
- [ ] Update desktop `StatusLine.svelte` to use `BaseStatusLine` with `VersionIndicator` slot
- [ ] Update web `StatusLine.svelte` to use `BaseStatusLine` (no extras)
- [ ] Update desktop `WinBar.svelte` to use `BaseWinBar` with extras slot
- [ ] Update web `WinBar.svelte` to use `BaseWinBar` (no extras)
- [ ] Run `bun run check` on all packages

### Phase 4: Full EditorLayout Extraction

- [ ] Create `packages/lib/src/components/editor-layout/EditorLayout.svelte`
- [ ] Migrate desktop `(editor)/+layout.svelte` to thin wrapper using `EditorLayout`
- [ ] Migrate web `(editor)/+layout.svelte` to thin wrapper using `EditorLayout`
- [ ] Verify snippets (`headerExtra`, `statusExtra`, `afterLayout`) work correctly
- [ ] Test focus/visibility refresh on both platforms
- [ ] Run full test suite: `bun run check && bun run tauri dev`

### Phase 5: Drift Prevention (Optional)

- [ ] Create `.github/workflows/check-drift.yml` for CI drift detection
- [ ] Or create `scripts/check-drift.ts` for local drift checking
- [ ] Add to CI pipeline
- [ ] Document expected differences in `ALLOWED_DIFFERENT` list

### Post-Implementation Verification

- [ ] Desktop: Window controls work (minimize, maximize, close)
- [ ] Desktop: Quick capture opens correctly
- [ ] Desktop: Updater dialog shows when update available
- [ ] Desktop: Drag region works for window movement
- [ ] Desktop: Focus refresh works after quick capture
- [ ] Web: Visibility refresh works on tab switch
- [ ] Web: No desktop-specific UI elements appear
- [ ] Both: Sidebar toggle works
- [ ] Both: Draft creation/editing/deletion works
- [ ] Both: Vim mode indicator shows correct mode

---

## Additional Patterns to Review

The following patterns in the codebase should be reviewed for potential extraction or improvement:

### 1. API Index Files (High Value)

`packages/{desktop,web}/src/lib/api/index.ts` contain nearly identical wrapper functions:

```typescript
// Both files have this same pattern
export async function listDrafts(): Promise<Draft[]> {
  const data = await backend.list();
  return data.map((d) => new Draft(d));
}
```

**Recommendation**: Extract to shared lib with platform backend injection.

### 2. Draft Page Routes

`packages/{desktop,web}/src/routes/(editor)/drafts/[[id]]/+page.svelte` should be compared for drift.

### 3. Load Functions

`packages/{desktop,web}/src/routes/(editor)/drafts/[[id]]/+page.ts` load functions should be compared.

### 4. Sidebar Context Pattern

The sidebar uses `Symbol.for()` pattern from shadcn-svelte:

```typescript
const SYMBOL_KEY = "scn-sidebar";
export function setSidebar(props: SidebarStateProps): SidebarState {
  return setContext(Symbol.for(SYMBOL_KEY), new SidebarState(props));
}
```

**Recommendation**: This is a third-party pattern (shadcn-svelte) and should remain as-is unless the upstream library changes. Document this exception.

### 5. Capture Component Vim Integration

The capture component uses global vim command registration with a module-level flag:

```typescript
let vimCommandsRegistered = false;
// ...
if (!vimCommandsRegistered) {
  Vim.defineEx('captureclose', ...);
  vimCommandsRegistered = true;
}
```

**Recommendation**: This pattern works but could be reviewed for potential improvements using Svelte's lifecycle or context patterns.
