# Monorepo Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split the Dashtext codebase into a bun workspace monorepo with four packages: `@dashtext/desktop`, `@dashtext/web`, `@dashtext/ui`, and `@dashtext/lib`.

**Architecture:** The desktop and web apps become independent SvelteKit applications, each owning their platform-specific code (routes, backends, platform features). Shared UI primitives live in `@dashtext/ui`. Shared business logic (types, stores, editor) lives in `@dashtext/lib`. No cross-imports between desktop and web - they only import from ui and lib.

**Tech Stack:** Bun workspaces, SvelteKit, Svelte 5, Tailwind CSS v4, Tauri v2, Cloudflare Workers

---

## Package Structure Overview

```
dashtext/
├── packages/
│   ├── desktop/           # @dashtext/desktop - Tauri app
│   │   ├── src/
│   │   │   ├── lib/
│   │   │   │   ├── api/
│   │   │   │   │   └── backend.ts      # Tauri/SQLite DraftAPI implementation
│   │   │   │   ├── db/                 # Drizzle schema + migrations
│   │   │   │   ├── components/
│   │   │   │   │   ├── updater/        # Desktop-only updater
│   │   │   │   │   └── capture/        # Quick capture window
│   │   │   │   └── stores/
│   │   │   │       └── updater.svelte.ts
│   │   │   └── routes/                 # Desktop routes
│   │   ├── src-tauri/                  # Rust backend
│   │   ├── static/
│   │   ├── package.json
│   │   ├── svelte.config.js
│   │   ├── vite.config.js
│   │   └── tsconfig.json
│   │
│   ├── web/               # @dashtext/web - Cloudflare Workers app
│   │   ├── src/
│   │   │   ├── lib/
│   │   │   │   └── api/
│   │   │   │       ├── backend.ts      # IndexedDB DraftAPI implementation
│   │   │   │       └── mock.ts         # Test mock
│   │   │   └── routes/                 # Web routes (simpler, no window controls)
│   │   ├── static/
│   │   ├── package.json
│   │   ├── svelte.config.js
│   │   ├── vite.config.js
│   │   ├── wrangler.jsonc
│   │   └── tsconfig.json
│   │
│   ├── ui/                # @dashtext/ui - Shared UI primitives
│   │   ├── src/
│   │   │   ├── components/
│   │   │   │   ├── button/
│   │   │   │   ├── dialog/
│   │   │   │   ├── sidebar/
│   │   │   │   ├── appbar/
│   │   │   │   └── ...
│   │   │   ├── styles/
│   │   │   │   ├── base.css            # Tailwind, theme variables
│   │   │   │   ├── codemirror.css
│   │   │   │   ├── layout.css
│   │   │   │   └── tokyonight_*.css
│   │   │   ├── utils.ts
│   │   │   └── index.ts
│   │   └── package.json
│   │
│   └── lib/               # @dashtext/lib - Shared business logic
│       ├── src/
│       │   ├── api/
│       │   │   └── types.ts            # DraftAPI interface, DraftData type
│       │   ├── stores/
│       │   │   └── drafts.svelte.ts    # Draft model, DraftsState
│       │   ├── editor/
│       │   │   ├── Editor.svelte
│       │   │   ├── context.svelte.ts
│       │   │   ├── extensions.ts
│       │   │   ├── codemirror-theme.ts
│       │   │   └── VimModeIndicator.svelte
│       │   └── index.ts
│       └── package.json
│
├── package.json           # Workspace root
├── tsconfig.json          # Shared TS config
└── ...                    # Docs, CI, etc.
```

---

## Phase 1: Workspace Foundation

**Goal:** Set up bun workspace infrastructure without moving any code yet.

### Task 1.1: Create packages directory structure

**Files:**
- Create: `packages/desktop/.gitkeep`
- Create: `packages/web/.gitkeep`
- Create: `packages/ui/.gitkeep`
- Create: `packages/lib/.gitkeep`

**Step 1:** Create the packages directory and subdirectories

```bash
mkdir -p packages/desktop packages/web packages/ui packages/lib
touch packages/desktop/.gitkeep packages/web/.gitkeep packages/ui/.gitkeep packages/lib/.gitkeep
```

**Step 2:** Commit

```bash
git add packages/
git commit -m "chore: create packages directory structure for monorepo"
```

---

### Task 1.2: Configure root package.json for workspaces

**Files:**
- Modify: `package.json`

**Step 1:** Update root package.json

Replace the entire contents with:

```json
{
  "name": "dashtext",
  "private": true,
  "workspaces": ["packages/*"],
  "scripts": {
    "dev:desktop": "bun --filter @dashtext/desktop dev",
    "dev:web": "bun --filter @dashtext/web dev",
    "build:desktop": "bun --filter @dashtext/desktop build",
    "build:web": "bun --filter @dashtext/web build",
    "tauri": "bun --filter @dashtext/desktop tauri",
    "check": "bun --filter '*' check",
    "screenshots": "bun --filter @dashtext/desktop screenshots"
  },
  "devDependencies": {
    "typescript": "~5.9.3"
  }
}
```

**Step 2:** Commit

```bash
git add package.json
git commit -m "chore: configure bun workspaces in root package.json"
```

---

### Task 1.3: Create shared TypeScript config

**Files:**
- Create: `tsconfig.base.json`
- Modify: `tsconfig.json`

**Step 1:** Create base tsconfig that packages will extend

Create `tsconfig.base.json`:

```json
{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler",
    "module": "ESNext",
    "target": "ESNext"
  }
}
```

**Step 2:** Update root tsconfig.json to reference base

Replace `tsconfig.json`:

```json
{
  "extends": "./tsconfig.base.json",
  "compilerOptions": {
    "plugins": [
      {
        "name": "typescript-svelte-plugin"
      }
    ]
  },
  "references": [
    { "path": "./packages/lib" },
    { "path": "./packages/ui" },
    { "path": "./packages/desktop" },
    { "path": "./packages/web" }
  ]
}
```

**Step 3:** Commit

```bash
git add tsconfig.base.json tsconfig.json
git commit -m "chore: set up shared TypeScript configuration"
```

---

## Phase 2: @dashtext/ui Package

**Goal:** Extract shared UI components (shadcn-svelte) and styles into `@dashtext/ui`.

### Task 2.1: Create @dashtext/ui package.json

**Files:**
- Create: `packages/ui/package.json`
- Remove: `packages/ui/.gitkeep`

**Step 1:** Create package.json

```json
{
  "name": "@dashtext/ui",
  "version": "0.0.1",
  "type": "module",
  "svelte": "./src/index.ts",
  "exports": {
    ".": {
      "types": "./src/index.ts",
      "svelte": "./src/index.ts",
      "default": "./src/index.ts"
    },
    "./styles/*.css": "./src/styles/*.css",
    "./button": {
      "types": "./src/components/button/index.ts",
      "svelte": "./src/components/button/index.ts"
    },
    "./dialog": {
      "types": "./src/components/dialog/index.ts",
      "svelte": "./src/components/dialog/index.ts"
    },
    "./input": {
      "types": "./src/components/input/index.ts",
      "svelte": "./src/components/input/index.ts"
    },
    "./separator": {
      "types": "./src/components/separator/index.ts",
      "svelte": "./src/components/separator/index.ts"
    },
    "./sheet": {
      "types": "./src/components/sheet/index.ts",
      "svelte": "./src/components/sheet/index.ts"
    },
    "./sidebar": {
      "types": "./src/components/sidebar/index.ts",
      "svelte": "./src/components/sidebar/index.ts"
    },
    "./skeleton": {
      "types": "./src/components/skeleton/index.ts",
      "svelte": "./src/components/skeleton/index.ts"
    },
    "./toggle": {
      "types": "./src/components/toggle/index.ts",
      "svelte": "./src/components/toggle/index.ts"
    },
    "./tooltip": {
      "types": "./src/components/tooltip/index.ts",
      "svelte": "./src/components/tooltip/index.ts"
    },
    "./appbar": {
      "types": "./src/components/appbar/index.ts",
      "svelte": "./src/components/appbar/index.ts"
    },
    "./utils": {
      "types": "./src/utils.ts",
      "default": "./src/utils.ts"
    }
  },
  "files": ["src"],
  "dependencies": {
    "bits-ui": "^2.14.4",
    "clsx": "^2.1.1",
    "tailwind-merge": "^3.4.0",
    "tailwind-variants": "^3.2.2"
  },
  "peerDependencies": {
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "svelte": "^5.45.3",
    "@lucide/svelte": "^0.555.0"
  }
}
```

**Step 2:** Remove .gitkeep

```bash
rm packages/ui/.gitkeep
```

**Step 3:** Commit

```bash
git add packages/ui/package.json
git rm packages/ui/.gitkeep
git commit -m "chore(ui): initialize @dashtext/ui package"
```

---

### Task 2.2: Move UI components to @dashtext/ui

**Files:**
- Create: `packages/ui/src/components/` (entire directory tree)
- Create: `packages/ui/src/utils.ts`
- Create: `packages/ui/src/index.ts`

**Step 1:** Create src directory structure

```bash
mkdir -p packages/ui/src/components
```

**Step 2:** Copy UI components

```bash
cp -r src/lib/components/ui/* packages/ui/src/components/
cp -r src/lib/components/appbar packages/ui/src/components/
```

**Step 3:** Copy and adapt utils.ts

Create `packages/ui/src/utils.ts`:

```typescript
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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

**Step 4:** Create main index.ts

Create `packages/ui/src/index.ts`:

```typescript
// Re-export utilities
export * from "./utils";

// Re-export all components
export * as Button from "./components/button";
export * as Dialog from "./components/dialog";
export * as Input from "./components/input";
export * as Separator from "./components/separator";
export * as Sheet from "./components/sheet";
export * as Sidebar from "./components/sidebar";
export * as Skeleton from "./components/skeleton";
export * as Toggle from "./components/toggle";
export * as Tooltip from "./components/tooltip";
export * as AppBar from "./components/appbar";
```

**Step 5:** Update imports in copied components

All components that import from `$lib/utils.js` need to import from `../utils.js` or `../../utils.js` instead.

Files to update (change `$lib/utils.js` to relative path):
- `packages/ui/src/components/button/button.svelte` → `import { cn, type WithElementRef } from "../../utils.js";`
- `packages/ui/src/components/appbar/AppBar.svelte` → `import { cn, type WithElementRef } from "../../utils.js";`
- `packages/ui/src/components/appbar/AppBarSection.svelte` → `import { cn, type WithElementRef } from "../../utils.js";`
- (And all other components that use `$lib/utils.js`)

**Step 6:** Commit

```bash
git add packages/ui/src/
git commit -m "feat(ui): add UI components and utilities to @dashtext/ui"
```

---

### Task 2.3: Move styles to @dashtext/ui

**Files:**
- Create: `packages/ui/src/styles/base.css`
- Create: `packages/ui/src/styles/codemirror.css`
- Create: `packages/ui/src/styles/layout.css`
- Create: `packages/ui/src/styles/tokyonight_day.css`
- Create: `packages/ui/src/styles/tokyonight_moon.css`

**Step 1:** Create styles directory

```bash
mkdir -p packages/ui/src/styles
```

**Step 2:** Copy style files

```bash
cp src/styles/codemirror.css packages/ui/src/styles/
cp src/styles/layout.css packages/ui/src/styles/
cp src/styles/tokyonight_day.css packages/ui/src/styles/
cp src/styles/tokyonight_moon.css packages/ui/src/styles/
```

**Step 3:** Create base.css (extracted from app.css)

Create `packages/ui/src/styles/base.css`:

```css
@import "tailwindcss";
@import "tw-animate-css";
@plugin '@tailwindcss/forms';
@plugin '@tailwindcss/typography';

:root {
  --radius: 0;
  --background: oklch(1 0 0);
  --foreground: oklch(0.141 0.005 285.823);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.141 0.005 285.823);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.141 0.005 285.823);
  --primary: oklch(0.21 0.006 285.885);
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.967 0.001 286.375);
  --secondary-foreground: oklch(0.21 0.006 285.885);
  --muted: oklch(0.967 0.001 286.375);
  --muted-foreground: oklch(0.552 0.016 285.938);
  --accent: oklch(0.967 0.001 286.375);
  --accent-foreground: oklch(0.21 0.006 285.885);
  --destructive: oklch(0.577 0.245 27.325);
  --border: oklch(0.92 0.004 286.32);
  --input: oklch(0.92 0.004 286.32);
  --ring: oklch(0.705 0.015 286.067);
  --chart-1: oklch(0.646 0.222 41.116);
  --chart-2: oklch(0.6 0.118 184.704);
  --chart-3: oklch(0.398 0.07 227.392);
  --chart-4: oklch(0.828 0.189 84.429);
  --chart-5: oklch(0.769 0.188 70.08);
  --sidebar: oklch(0.985 0 0);
  --sidebar-foreground: oklch(0.141 0.005 285.823);
  --sidebar-primary: oklch(0.21 0.006 285.885);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.967 0.001 286.375);
  --sidebar-accent-foreground: oklch(0.21 0.006 285.885);
  --sidebar-border: oklch(0.92 0.004 286.32);
  --sidebar-ring: oklch(0.705 0.015 286.067);
}

.dark {
  --background: oklch(0.141 0.005 285.823);
  --foreground: oklch(0.985 0 0);
  --card: oklch(0.21 0.006 285.885);
  --card-foreground: oklch(0.985 0 0);
  --popover: oklch(0.21 0.006 285.885);
  --popover-foreground: oklch(0.985 0 0);
  --primary: oklch(0.92 0.004 286.32);
  --primary-foreground: oklch(0.21 0.006 285.885);
  --secondary: oklch(0.274 0.006 286.033);
  --secondary-foreground: oklch(0.985 0 0);
  --muted: oklch(0.274 0.006 286.033);
  --muted-foreground: oklch(0.705 0.015 286.067);
  --accent: oklch(0.274 0.006 286.033);
  --accent-foreground: oklch(0.985 0 0);
  --destructive: oklch(0.704 0.191 22.216);
  --border: oklch(1 0 0 / 10%);
  --input: oklch(1 0 0 / 15%);
  --ring: oklch(0.552 0.016 285.938);
  --chart-1: oklch(0.488 0.243 264.376);
  --chart-2: oklch(0.696 0.17 162.48);
  --chart-3: oklch(0.769 0.188 70.08);
  --chart-4: oklch(0.627 0.265 303.9);
  --chart-5: oklch(0.645 0.246 16.439);
  --sidebar: oklch(0.21 0.006 285.885);
  --sidebar-foreground: oklch(0.985 0 0);
  --sidebar-primary: oklch(0.488 0.243 264.376);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.274 0.006 286.033);
  --sidebar-accent-foreground: oklch(0.985 0 0);
  --sidebar-border: oklch(1 0 0 / 10%);
  --sidebar-ring: oklch(0.552 0.016 285.938);
}

@theme inline {
  --radius-sm: calc(var(--radius) - 4px);
  --radius-md: calc(var(--radius) - 2px);
  --radius-lg: var(--radius);
  --radius-xl: calc(var(--radius) + 4px);
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --color-chart-1: var(--chart-1);
  --color-chart-2: var(--chart-2);
  --color-chart-3: var(--chart-3);
  --color-chart-4: var(--chart-4);
  --color-chart-5: var(--chart-5);
  --color-sidebar: var(--sidebar);
  --color-sidebar-foreground: var(--sidebar-foreground);
  --color-sidebar-primary: var(--sidebar-primary);
  --color-sidebar-primary-foreground: var(--sidebar-primary-foreground);
  --color-sidebar-accent: var(--sidebar-accent);
  --color-sidebar-accent-foreground: var(--sidebar-accent-foreground);
  --color-sidebar-border: var(--sidebar-border);
  --color-sidebar-ring: var(--sidebar-ring);
}

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

@layer base {
  * {
    @apply border-border outline-ring/50;
  }

  html,
  body {
    @apply h-full w-full overflow-hidden;
  }
}

@custom-variant dark (&:is(.dark *));
```

**Step 4:** Commit

```bash
git add packages/ui/src/styles/
git commit -m "feat(ui): add shared styles to @dashtext/ui"
```

---

## Phase 3: @dashtext/lib Package

**Goal:** Extract shared business logic (types, stores, editor) into `@dashtext/lib`.

### Task 3.1: Create @dashtext/lib package.json

**Files:**
- Create: `packages/lib/package.json`
- Create: `packages/lib/tsconfig.json`
- Remove: `packages/lib/.gitkeep`

**Step 1:** Create package.json

```json
{
  "name": "@dashtext/lib",
  "version": "0.0.1",
  "type": "module",
  "svelte": "./src/index.ts",
  "exports": {
    ".": {
      "types": "./src/index.ts",
      "svelte": "./src/index.ts",
      "default": "./src/index.ts"
    },
    "./api": {
      "types": "./src/api/index.ts",
      "default": "./src/api/index.ts"
    },
    "./stores": {
      "types": "./src/stores/index.ts",
      "svelte": "./src/stores/index.ts"
    },
    "./editor": {
      "types": "./src/editor/index.ts",
      "svelte": "./src/editor/index.ts"
    }
  },
  "files": ["src"],
  "dependencies": {
    "@codemirror/commands": "^6.10.0",
    "@codemirror/lang-markdown": "^6.5.0",
    "@codemirror/language": "^6.0.0",
    "@codemirror/language-data": "^6.5.2",
    "@codemirror/state": "^6.5.2",
    "@codemirror/view": "^6.38.8",
    "@lezer/highlight": "^1.2.3",
    "@replit/codemirror-vim": "^6.3.0",
    "runed": "^0.37.0"
  },
  "peerDependencies": {
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "svelte": "^5.45.3"
  }
}
```

**Step 2:** Create tsconfig.json

```json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "rootDir": "./src",
    "outDir": "./dist",
    "composite": true
  },
  "include": ["src/**/*"]
}
```

**Step 3:** Remove .gitkeep

```bash
rm packages/lib/.gitkeep
```

**Step 4:** Commit

```bash
git add packages/lib/package.json packages/lib/tsconfig.json
git rm packages/lib/.gitkeep
git commit -m "chore(lib): initialize @dashtext/lib package"
```

---

### Task 3.2: Move API types to @dashtext/lib

**Files:**
- Create: `packages/lib/src/api/types.ts`
- Create: `packages/lib/src/api/index.ts`

**Step 1:** Create api directory

```bash
mkdir -p packages/lib/src/api
```

**Step 2:** Create types.ts (copied from src/lib/api/types.ts)

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

**Step 3:** Create api/index.ts

```typescript
export * from "./types";
```

**Step 4:** Commit

```bash
git add packages/lib/src/api/
git commit -m "feat(lib): add DraftAPI types to @dashtext/lib"
```

---

### Task 3.3: Move stores to @dashtext/lib

**Files:**
- Create: `packages/lib/src/stores/drafts.svelte.ts`
- Create: `packages/lib/src/stores/index.ts`

**Step 1:** Create stores directory

```bash
mkdir -p packages/lib/src/stores
```

**Step 2:** Create drafts.svelte.ts

This needs to be adapted to not depend on `$lib/api` or `$app/navigation` directly. The API functions will be injected.

```typescript
import { createContext, untrack } from 'svelte';
import { useDebounce } from 'runed';
import type { DraftData } from '../api/types';

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

/**
 * API functions that DraftsState needs - injected by the app
 */
export interface DraftsAPI {
  createDraft(): Promise<Draft>;
  saveDraft(id: number, content: string): Promise<DraftData>;
  replaceUrl(url: string): void;
}

/**
 * DraftsState - unified state for all drafts and current draft with autosave
 * Set in context by root layout, consumed by Editor, Sidebar, and route pages
 */
export class DraftsState {
  drafts = $state<Draft[]>([]);
  currentDraft = $state<Draft | null>(null);

  private pendingContent: string | null = null;
  private debouncedSave: ReturnType<typeof useDebounce>;
  private api: DraftsAPI;

  constructor(initialDrafts: Draft[], api: DraftsAPI) {
    this.drafts = initialDrafts;
    this.api = api;
    this.debouncedSave = useDebounce(() => this.performSave(), () => 500);
  }

  setCurrentDraft(draft: Draft | null) {
    if (draft) {
      // Use existing draft from our array to maintain object identity for reactivity
      this.currentDraft = this.drafts.find((d) => d.id === draft.id) ?? draft;
    } else {
      this.currentDraft = null;
    }
  }

  updateContent(content: string) {
    if (this.currentDraft) {
      this.currentDraft.content = content;
    }
    this.pendingContent = content;
    this.debouncedSave();
  }

  flushPendingSave() {
    this.debouncedSave.runScheduledNow();
  }

  private async performSave() {
    if (this.pendingContent === null) return;

    if (this.currentDraft === null && this.pendingContent.trim()) {
      // NEW DRAFT: create in DB, update local state, replaceState URL
      const newDraft = await this.api.createDraft();
      newDraft.content = this.pendingContent;
      await this.api.saveDraft(newDraft.id, this.pendingContent);

      // Update local state (triggers reactive updates)
      this.drafts = [newDraft, ...this.drafts];
      this.currentDraft = newDraft;

      // Update URL without navigation (preserves focus)
      this.api.replaceUrl(`/drafts/${newDraft.id}`);
    } else if (this.currentDraft !== null) {
      // EXISTING DRAFT: just save
      const updated = await this.api.saveDraft(this.currentDraft.id, this.pendingContent);
      this.currentDraft.modified_at = updated.modified_at;
    }

    this.pendingContent = null;
  }
}

export const [getDraftsState, setDraftsState] = createContext<DraftsState>();

export const createDraftsState = (getInitialDrafts: () => Draft[], api: DraftsAPI) => {
  const drafts = new DraftsState(untrack(getInitialDrafts), api);
  setDraftsState(drafts);
  return drafts;
};
```

**Step 3:** Create stores/index.ts

```typescript
export * from "./drafts.svelte";
```

**Step 4:** Commit

```bash
git add packages/lib/src/stores/
git commit -m "feat(lib): add Draft model and DraftsState to @dashtext/lib"
```

---

### Task 3.4: Move editor components to @dashtext/lib

**Files:**
- Create: `packages/lib/src/editor/Editor.svelte`
- Create: `packages/lib/src/editor/context.svelte.ts`
- Create: `packages/lib/src/editor/extensions.ts`
- Create: `packages/lib/src/editor/codemirror-theme.ts`
- Create: `packages/lib/src/editor/VimModeIndicator.svelte`
- Create: `packages/lib/src/editor/index.ts`

**Step 1:** Create editor directory

```bash
mkdir -p packages/lib/src/editor
```

**Step 2:** Copy editor files

```bash
cp src/lib/components/editor/context.svelte.ts packages/lib/src/editor/
cp src/lib/components/editor/extensions.ts packages/lib/src/editor/
cp src/lib/components/editor/codemirror-theme.ts packages/lib/src/editor/
cp src/lib/components/editor/VimModeIndicator.svelte packages/lib/src/editor/
cp src/lib/components/editor/Editor.svelte packages/lib/src/editor/
```

**Step 3:** Update imports in copied files

In `packages/lib/src/editor/Editor.svelte`:
- Change `import { getDraftsState } from '$lib/stores/drafts.svelte';` to `import { getDraftsState } from '../stores/drafts.svelte';`

In `packages/lib/src/editor/extensions.ts`:
- The import `import { tokyoNightTheme } from './codemirror-theme';` is already correct (relative)

**Step 4:** Create editor/index.ts

```typescript
export { default as Editor } from './Editor.svelte';
export { default as VimModeIndicator } from './VimModeIndicator.svelte';
export { createEditorContext, getEditorContext, EditorState, type VimModeType } from './context.svelte';
export { createExtensions, createBaseExtensions } from './extensions';
export { tokyoNightTheme, tokyoNightEditorTheme, tokyoNightHighlightStyle } from './codemirror-theme';
```

**Step 5:** Commit

```bash
git add packages/lib/src/editor/
git commit -m "feat(lib): add editor components to @dashtext/lib"
```

---

### Task 3.5: Create @dashtext/lib main index

**Files:**
- Create: `packages/lib/src/index.ts`

**Step 1:** Create main index.ts

```typescript
// API types
export * from "./api";

// Stores
export * from "./stores";

// Editor
export * from "./editor";
```

**Step 2:** Commit

```bash
git add packages/lib/src/index.ts
git commit -m "feat(lib): add main exports for @dashtext/lib"
```

---

## Phase 4: @dashtext/desktop Package

**Goal:** Create the desktop Tauri application with its own routes and backend.

### Task 4.1: Create @dashtext/desktop package.json and configs

**Files:**
- Create: `packages/desktop/package.json`
- Create: `packages/desktop/svelte.config.js`
- Create: `packages/desktop/vite.config.js`
- Create: `packages/desktop/tsconfig.json`
- Remove: `packages/desktop/.gitkeep`

**Step 1:** Create package.json

```json
{
  "name": "@dashtext/desktop",
  "version": "0.3.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "tauri": "tauri",
    "screenshots": "playwright test --project=screenshots"
  },
  "dependencies": {
    "@dashtext/lib": "workspace:*",
    "@dashtext/ui": "workspace:*",
    "@tauri-apps/api": "^2.9.1",
    "@tauri-apps/plugin-opener": "^2.5.2",
    "@tauri-apps/plugin-sql": "^2.3.1",
    "drizzle-orm": "^0.44.7"
  },
  "devDependencies": {
    "@lucide/svelte": "^0.555.0",
    "@playwright/test": "^1.49.1",
    "@sveltejs/adapter-static": "^3.0.10",
    "@sveltejs/kit": "^2.49.1",
    "@sveltejs/vite-plugin-svelte": "^6.2.1",
    "@tailwindcss/forms": "^0.5.10",
    "@tailwindcss/typography": "^0.5.19",
    "@tailwindcss/vite": "^4.1.17",
    "@tauri-apps/cli": "^2.9.5",
    "drizzle-kit": "^0.31.7",
    "svelte": "^5.45.3",
    "svelte-check": "^4.3.4",
    "tailwindcss": "^4.1.17",
    "tw-animate-css": "^1.4.0",
    "typescript": "~5.9.3",
    "vite": "^7.2.6"
  }
}
```

**Step 2:** Create svelte.config.js

```javascript
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    alias: {
      "$lib": "./src/lib",
      "$lib/*": "./src/lib/*"
    }
  },
};

export default config;
```

**Step 3:** Create vite.config.js

```javascript
import { readFileSync } from "fs";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit()],
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"]
    }
  }
}));
```

**Step 4:** Create tsconfig.json

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler",
    "plugins": [
      {
        "name": "typescript-svelte-plugin"
      }
    ]
  }
}
```

**Step 5:** Remove .gitkeep

```bash
rm packages/desktop/.gitkeep
```

**Step 6:** Commit

```bash
git add packages/desktop/package.json packages/desktop/svelte.config.js packages/desktop/vite.config.js packages/desktop/tsconfig.json
git rm packages/desktop/.gitkeep
git commit -m "chore(desktop): initialize @dashtext/desktop package configs"
```

---

### Task 4.2: Move src-tauri to desktop package

**Files:**
- Move: `src-tauri/` → `packages/desktop/src-tauri/`

**Step 1:** Move src-tauri directory

```bash
mv src-tauri packages/desktop/
```

**Step 2:** Update tauri.conf.json paths

In `packages/desktop/src-tauri/tauri.conf.json`, update the `frontendDist` and `devUrl` paths if necessary. The existing config should work as-is since paths are relative.

**Step 3:** Commit

```bash
git add packages/desktop/src-tauri/
git commit -m "feat(desktop): move Tauri backend to @dashtext/desktop"
```

---

### Task 4.3: Create desktop lib directory structure

**Files:**
- Create: `packages/desktop/src/lib/api/backend.ts`
- Create: `packages/desktop/src/lib/api/index.ts`
- Create: `packages/desktop/src/lib/db/schema.ts`
- Create: `packages/desktop/src/lib/db/index.ts`
- Move: `src/lib/db/migrations/` → `packages/desktop/src/lib/db/migrations/`

**Step 1:** Create directory structure

```bash
mkdir -p packages/desktop/src/lib/api
mkdir -p packages/desktop/src/lib/db
```

**Step 2:** Copy database files

```bash
cp src/lib/db/schema.ts packages/desktop/src/lib/db/
cp src/lib/db/index.ts packages/desktop/src/lib/db/
cp -r src/lib/db/migrations packages/desktop/src/lib/db/
```

**Step 3:** Create backend.ts (Tauri/SQLite implementation)

```typescript
import { eq, desc } from 'drizzle-orm';
import { getDb, drafts } from '$lib/db';
import type { DraftAPI, DraftData, Draft } from '@dashtext/lib';
import type { Draft as DrizzleDraft } from '$lib/db/schema';

/**
 * Convert Drizzle row (camelCase) to DraftData (snake_case)
 */
function toApiFormat(row: DrizzleDraft): DraftData {
  return {
    id: row.id,
    content: row.content,
    created_at: row.createdAt,
    modified_at: row.modifiedAt,
  };
}

/**
 * Get current ISO timestamp string
 */
function getNow(): string {
  return new Date().toISOString();
}

const tauriBackend: DraftAPI = {
  async list(): Promise<DraftData[]> {
    const db = await getDb();
    const rows = await db.select().from(drafts).orderBy(desc(drafts.modifiedAt));
    return rows.map(toApiFormat);
  },

  async create(): Promise<DraftData> {
    const db = await getDb();
    const now = getNow();
    const result = await db.insert(drafts).values({
      content: '',
      createdAt: now,
      modifiedAt: now,
    }).returning();
    return toApiFormat(result[0]);
  },

  async get(id: number): Promise<DraftData | null> {
    const db = await getDb();
    const rows = await db.select().from(drafts).where(eq(drafts.id, id));
    return rows.length > 0 ? toApiFormat(rows[0]) : null;
  },

  async save(id: number, content: string): Promise<DraftData> {
    const db = await getDb();
    const now = getNow();
    const result = await db.update(drafts)
      .set({ content, modifiedAt: now })
      .where(eq(drafts.id, id))
      .returning();
    return toApiFormat(result[0]);
  },

  async delete(id: number): Promise<void> {
    const db = await getDb();
    await db.delete(drafts).where(eq(drafts.id, id));
  },
};

export default tauriBackend;
```

**Step 4:** Create api/index.ts

```typescript
import { Draft } from '@dashtext/lib';
import type { DraftData } from '@dashtext/lib';
import backend from './backend';

export type { DraftAPI, DraftData } from '@dashtext/lib';

export async function listDrafts(): Promise<Draft[]> {
  const data = await backend.list();
  return data.map((d) => new Draft(d));
}

export async function createDraft(): Promise<Draft> {
  const data = await backend.create();
  return new Draft(data);
}

export async function getDraft(id: number): Promise<Draft | null> {
  const data = await backend.get(id);
  return data ? new Draft(data) : null;
}

export async function saveDraft(id: number, content: string): Promise<DraftData> {
  return backend.save(id, content);
}

export async function deleteDraft(id: number): Promise<void> {
  return backend.delete(id);
}
```

**Step 5:** Commit

```bash
git add packages/desktop/src/lib/
git commit -m "feat(desktop): add Tauri backend and database to @dashtext/desktop"
```

---

### Task 4.4: Move desktop-specific components

**Files:**
- Create: `packages/desktop/src/lib/stores/updater.svelte.ts`
- Create: `packages/desktop/src/lib/components/updater/`
- Create: `packages/desktop/src/lib/components/capture/`

**Step 1:** Create directories

```bash
mkdir -p packages/desktop/src/lib/stores
mkdir -p packages/desktop/src/lib/components/updater
mkdir -p packages/desktop/src/lib/components/capture
```

**Step 2:** Copy updater store and components

```bash
cp src/lib/stores/updater.svelte.ts packages/desktop/src/lib/stores/
cp -r src/lib/components/updater/* packages/desktop/src/lib/components/updater/
```

**Step 3:** Copy capture components

```bash
cp -r src/lib/components/capture/* packages/desktop/src/lib/components/capture/
```

**Step 4:** Update imports in copied files

In updater components, update imports from `$lib/components/ui/` to `@dashtext/ui/`:
- `VersionIndicator.svelte`: `import * as Tooltip from '@dashtext/ui/tooltip';`
- `UpdateDialog.svelte`: Similar updates

In capture components, update imports:
- `CaptureEditor.svelte`: Update `$lib/components/editor/context.svelte` to `@dashtext/lib/editor`
- `extensions.ts`: Update `$lib/components/editor/extensions` to `@dashtext/lib/editor`

**Step 5:** Commit

```bash
git add packages/desktop/src/lib/stores/ packages/desktop/src/lib/components/
git commit -m "feat(desktop): add desktop-specific stores and components"
```

---

### Task 4.5: Create desktop routes

**Files:**
- Create: `packages/desktop/src/routes/+layout.svelte`
- Create: `packages/desktop/src/routes/+layout.ts`
- Create: `packages/desktop/src/routes/+page.ts`
- Create: `packages/desktop/src/routes/(editor)/+layout.svelte`
- Create: `packages/desktop/src/routes/(editor)/+layout.ts`
- Create: `packages/desktop/src/routes/(editor)/WinBar.svelte`
- Create: `packages/desktop/src/routes/(editor)/StatusLine.svelte`
- Create: `packages/desktop/src/routes/(editor)/WindowControls.svelte`
- Create: `packages/desktop/src/routes/(editor)/drafts/[[id]]/+page.svelte`
- Create: `packages/desktop/src/routes/(editor)/drafts/[[id]]/+page.ts`
- Create: `packages/desktop/src/routes/capture/+layout.svelte`
- Create: `packages/desktop/src/routes/capture/+page.svelte`

**Step 1:** Create routes directory structure

```bash
mkdir -p packages/desktop/src/routes/\(editor\)/drafts/\[\[id\]\]
mkdir -p packages/desktop/src/routes/capture
```

**Step 2:** Copy and adapt route files

Copy all route files from `src/routes/` to `packages/desktop/src/routes/`.

Key changes needed in copied files:
- Remove all `isTauri()` checks - this IS the desktop app
- Update imports from `$lib/components/ui/` to `@dashtext/ui/`
- Update imports from `$lib/components/editor` to `@dashtext/lib/editor`
- Update imports from `$lib/stores/drafts.svelte` to `@dashtext/lib/stores`
- Keep `$lib/api` as is (local to desktop package)
- Keep `$lib/stores/updater.svelte` as is (local to desktop package)
- Keep `$lib/components/updater` as is (local to desktop package)
- Keep `$lib/components/capture` as is (local to desktop package)

**Step 3:** Commit

```bash
git add packages/desktop/src/routes/
git commit -m "feat(desktop): add desktop routes to @dashtext/desktop"
```

---

### Task 4.6: Create desktop app entry files

**Files:**
- Create: `packages/desktop/src/app.html`
- Create: `packages/desktop/src/app.css`
- Create: `packages/desktop/src/app.d.ts`
- Create: `packages/desktop/static/favicon.png`

**Step 1:** Copy app files

```bash
cp src/app.html packages/desktop/src/
cp src/app.d.ts packages/desktop/src/
mkdir -p packages/desktop/static
cp static/favicon.png packages/desktop/static/
cp -r src-tauri/icons packages/desktop/static/ 2>/dev/null || true
```

**Step 2:** Create app.css that imports from @dashtext/ui

Create `packages/desktop/src/app.css`:

```css
@import "@dashtext/ui/styles/base.css";
@import "@dashtext/ui/styles/codemirror.css";
@import "@dashtext/ui/styles/layout.css";
@import "@dashtext/ui/styles/tokyonight_day.css";
@import "@dashtext/ui/styles/tokyonight_moon.css";
```

**Step 3:** Commit

```bash
git add packages/desktop/src/app.html packages/desktop/src/app.css packages/desktop/src/app.d.ts packages/desktop/static/
git commit -m "feat(desktop): add app entry files to @dashtext/desktop"
```

---

### Task 4.7: Move drizzle config and playwright to desktop

**Files:**
- Move: `drizzle.config.ts` → `packages/desktop/drizzle.config.ts`
- Move: `playwright.config.js` → `packages/desktop/playwright.config.js`
- Move: `test/` → `packages/desktop/test/`

**Step 1:** Move config files

```bash
mv drizzle.config.ts packages/desktop/
mv playwright.config.js packages/desktop/
mv test packages/desktop/
```

**Step 2:** Update drizzle.config.ts paths

Update `packages/desktop/drizzle.config.ts` to use correct relative paths:

```typescript
import { defineConfig } from 'drizzle-kit';

export default defineConfig({
  schema: './src/lib/db/schema.ts',
  out: './src/lib/db/migrations',
  dialect: 'sqlite',
});
```

**Step 3:** Commit

```bash
git add packages/desktop/drizzle.config.ts packages/desktop/playwright.config.js packages/desktop/test/
git commit -m "feat(desktop): move drizzle and playwright config to @dashtext/desktop"
```

---

## Phase 5: @dashtext/web Package

**Goal:** Create the web Cloudflare Workers application with its own routes and backend.

### Task 5.1: Create @dashtext/web package.json and configs

**Files:**
- Create: `packages/web/package.json`
- Create: `packages/web/svelte.config.js`
- Create: `packages/web/vite.config.js`
- Create: `packages/web/tsconfig.json`
- Move: `wrangler.jsonc` → `packages/web/wrangler.jsonc`
- Remove: `packages/web/.gitkeep`

**Step 1:** Create package.json

```json
{
  "name": "@dashtext/web",
  "version": "0.3.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "deploy": "vite build && wrangler deploy"
  },
  "dependencies": {
    "@dashtext/lib": "workspace:*",
    "@dashtext/ui": "workspace:*",
    "idb": "^8.0.3"
  },
  "devDependencies": {
    "@lucide/svelte": "^0.555.0",
    "@sveltejs/adapter-cloudflare": "^7.2.4",
    "@sveltejs/kit": "^2.49.1",
    "@sveltejs/vite-plugin-svelte": "^6.2.1",
    "@tailwindcss/forms": "^0.5.10",
    "@tailwindcss/typography": "^0.5.19",
    "@tailwindcss/vite": "^4.1.17",
    "svelte": "^5.45.3",
    "svelte-check": "^4.3.4",
    "tailwindcss": "^4.1.17",
    "tw-animate-css": "^1.4.0",
    "typescript": "~5.9.3",
    "vite": "^7.2.6",
    "wrangler": "^4.22.0"
  }
}
```

**Step 2:** Create svelte.config.js

```javascript
import adapter from "@sveltejs/adapter-cloudflare";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    alias: {
      "$lib": "./src/lib",
      "$lib/*": "./src/lib/*"
    }
  },
};

export default config;
```

**Step 3:** Create vite.config.js

```javascript
import { readFileSync } from "fs";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit()],
  define: { __APP_VERSION__: JSON.stringify(pkg.version) }
}));
```

**Step 4:** Create tsconfig.json

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler",
    "plugins": [
      {
        "name": "typescript-svelte-plugin"
      }
    ]
  }
}
```

**Step 5:** Move wrangler.jsonc

```bash
mv wrangler.jsonc packages/web/
```

**Step 6:** Remove .gitkeep

```bash
rm packages/web/.gitkeep
```

**Step 7:** Commit

```bash
git add packages/web/package.json packages/web/svelte.config.js packages/web/vite.config.js packages/web/tsconfig.json packages/web/wrangler.jsonc
git rm packages/web/.gitkeep
git commit -m "chore(web): initialize @dashtext/web package configs"
```

---

### Task 5.2: Create web lib directory with IndexedDB backend

**Files:**
- Create: `packages/web/src/lib/api/backend.ts`
- Create: `packages/web/src/lib/api/mock.ts`
- Create: `packages/web/src/lib/api/index.ts`

**Step 1:** Create directory structure

```bash
mkdir -p packages/web/src/lib/api
```

**Step 2:** Create backend.ts (IndexedDB implementation)

Copy and adapt from `src/lib/api/backends/indexeddb.ts`:

```typescript
import { openDB, type DBSchema, type IDBPDatabase } from 'idb';
import type { DraftAPI, DraftData } from '@dashtext/lib';

interface DashtextDB extends DBSchema {
  drafts: {
    key: number;
    value: DraftData;
    indexes: {
      'by-modified': string;
    };
  };
}

const DB_NAME = 'dashtext';
const DB_VERSION = 1;

let dbPromise: Promise<IDBPDatabase<DashtextDB>> | null = null;

function getDB(): Promise<IDBPDatabase<DashtextDB>> {
  if (!dbPromise) {
    dbPromise = openDB<DashtextDB>(DB_NAME, DB_VERSION, {
      upgrade(db) {
        const store = db.createObjectStore('drafts', {
          keyPath: 'id',
          autoIncrement: true
        });
        store.createIndex('by-modified', 'modified_at');
      }
    });
  }
  return dbPromise;
}

class IndexedDBBackend implements DraftAPI {
  async list(): Promise<DraftData[]> {
    const db = await getDB();
    const drafts = await db.getAll('drafts');
    return drafts.sort(
      (a, b) => new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime()
    );
  }

  async create(): Promise<DraftData> {
    const db = await getDB();
    const now = new Date().toISOString();
    const draft: Omit<DraftData, 'id'> = {
      content: '',
      created_at: now,
      modified_at: now
    };

    const id = await db.add('drafts', draft as DraftData);
    return { ...draft, id: id as number };
  }

  async get(id: number): Promise<DraftData | null> {
    const db = await getDB();
    const draft = await db.get('drafts', id);
    return draft ?? null;
  }

  async save(id: number, content: string): Promise<DraftData> {
    const db = await getDB();
    const draft = await db.get('drafts', id);

    if (!draft) {
      throw new Error(`Draft ${id} not found`);
    }

    const updated: DraftData = {
      ...draft,
      content,
      modified_at: new Date().toISOString()
    };

    await db.put('drafts', updated);
    return updated;
  }

  async delete(id: number): Promise<void> {
    const db = await getDB();
    await db.delete('drafts', id);
  }
}

export default new IndexedDBBackend();
```

**Step 3:** Create mock.ts (for testing)

Copy from `src/lib/api/backends/mock.ts`:

```typescript
import type { DraftAPI, DraftData } from '@dashtext/lib';

let nextId = 1;
const drafts: Map<number, DraftData> = new Map();

function getNow(): string {
  return new Date().toISOString();
}

export const mockBackend: DraftAPI = {
  async list(): Promise<DraftData[]> {
    return Array.from(drafts.values()).sort(
      (a, b) => new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime()
    );
  },

  async create(): Promise<DraftData> {
    const now = getNow();
    const draft: DraftData = {
      id: nextId++,
      content: '',
      created_at: now,
      modified_at: now
    };
    drafts.set(draft.id, draft);
    return draft;
  },

  async get(id: number): Promise<DraftData | null> {
    return drafts.get(id) ?? null;
  },

  async save(id: number, content: string): Promise<DraftData> {
    const draft = drafts.get(id);
    if (!draft) throw new Error(`Draft ${id} not found`);

    const updated = { ...draft, content, modified_at: getNow() };
    drafts.set(id, updated);
    return updated;
  },

  async delete(id: number): Promise<void> {
    drafts.delete(id);
  }
};

export function resetMockBackend(): void {
  drafts.clear();
  nextId = 1;
}
```

**Step 4:** Create api/index.ts

```typescript
import { Draft } from '@dashtext/lib';
import type { DraftData } from '@dashtext/lib';
import backend from './backend';

export type { DraftAPI, DraftData } from '@dashtext/lib';

export async function listDrafts(): Promise<Draft[]> {
  const data = await backend.list();
  return data.map((d) => new Draft(d));
}

export async function createDraft(): Promise<Draft> {
  const data = await backend.create();
  return new Draft(data);
}

export async function getDraft(id: number): Promise<Draft | null> {
  const data = await backend.get(id);
  return data ? new Draft(data) : null;
}

export async function saveDraft(id: number, content: string): Promise<DraftData> {
  return backend.save(id, content);
}

export async function deleteDraft(id: number): Promise<void> {
  return backend.delete(id);
}
```

**Step 5:** Commit

```bash
git add packages/web/src/lib/
git commit -m "feat(web): add IndexedDB backend to @dashtext/web"
```

---

### Task 5.3: Create web routes

**Files:**
- Create: `packages/web/src/routes/+layout.svelte`
- Create: `packages/web/src/routes/+layout.ts`
- Create: `packages/web/src/routes/+page.ts`
- Create: `packages/web/src/routes/(editor)/+layout.svelte`
- Create: `packages/web/src/routes/(editor)/+layout.ts`
- Create: `packages/web/src/routes/(editor)/WinBar.svelte`
- Create: `packages/web/src/routes/(editor)/StatusLine.svelte`
- Create: `packages/web/src/routes/(editor)/drafts/[[id]]/+page.svelte`
- Create: `packages/web/src/routes/(editor)/drafts/[[id]]/+page.ts`
- Create: `packages/web/src/routes/capture/+layout.svelte`
- Create: `packages/web/src/routes/capture/+page.svelte`

**Step 1:** Create routes directory structure

```bash
mkdir -p packages/web/src/routes/\(editor\)/drafts/\[\[id\]\]
mkdir -p packages/web/src/routes/capture
```

**Step 2:** Create simplified web routes

The web routes are similar to desktop but:
- No `isTauri()` checks needed
- No WindowControls component
- No updater components
- Quick capture navigates to `/capture` route instead of opening a new window
- Simpler WinBar without window controls section

Create `packages/web/src/routes/(editor)/+layout.svelte`:

```svelte
<script lang="ts">
  import { createEditorContext } from '@dashtext/lib/editor';
  import * as Sidebar from '@dashtext/ui/sidebar';
  import { listDrafts } from '$lib/api';
  import { createDraftsState, type DraftsAPI } from '@dashtext/lib/stores';
  import { createDraft, saveDraft } from '$lib/api';
  import { replaceState } from '$app/navigation';
  import StatusLine from './StatusLine.svelte';
  import WinBar from './WinBar.svelte';

  let { data, children } = $props();

  createEditorContext();
  
  // Create the API adapter for DraftsState
  const draftsAPI: DraftsAPI = {
    createDraft: async () => createDraft(),
    saveDraft: async (id, content) => saveDraft(id, content),
    replaceUrl: (url) => replaceState(url, {})
  };
  
  const draftsState = createDraftsState(() => data.drafts, draftsAPI);

  // Refresh drafts when tab becomes visible
  $effect(() => {
    const handleVisibility = () => {
      if (document.visibilityState === 'visible') {
        listDrafts().then((fresh) => {
          draftsState.drafts = fresh;
        });
      }
    };

    document.addEventListener('visibilitychange', handleVisibility);
    return () => {
      document.removeEventListener('visibilitychange', handleVisibility);
    };
  });
</script>

<Sidebar.Provider>
  <div data-layout="root">
    <WinBar />
    <Sidebar.Root collapsible="offcanvas">
      <Sidebar.Content class="gap-0">
        {#if draftsState.currentDraft === null}
          <a
            href="/drafts/new"
            class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors bg-sidebar-accent"
          >
            <div class="truncate text-sm font-medium text-sidebar-foreground">
              Untitled
            </div>
          </a>
        {/if}
        {#each draftsState.drafts as draft (draft.id)}
          <a
            href="/drafts/{draft.id}"
            class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
            class:bg-sidebar-accent={draftsState.currentDraft?.id === draft.id}
          >
          <div class="w-full truncate text-sm font-medium text-sidebar-foreground">
            {draft.title}
          </div>
          {#each draft.previewLines as line, i (i)}
            <div class="w-full truncate text-xs text-sidebar-foreground/60">
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

**Step 3:** Create other route files similarly

The other route files follow the same pattern - copy from desktop and remove Tauri-specific code.

**Step 4:** Commit

```bash
git add packages/web/src/routes/
git commit -m "feat(web): add web routes to @dashtext/web"
```

---

### Task 5.4: Create web app entry files

**Files:**
- Create: `packages/web/src/app.html`
- Create: `packages/web/src/app.css`
- Create: `packages/web/src/app.d.ts`
- Create: `packages/web/static/favicon.png`

**Step 1:** Copy app files

```bash
cp src/app.html packages/web/src/
cp src/app.d.ts packages/web/src/
mkdir -p packages/web/static
cp static/favicon.png packages/web/static/
```

**Step 2:** Create app.css

Create `packages/web/src/app.css`:

```css
@import "@dashtext/ui/styles/base.css";
@import "@dashtext/ui/styles/codemirror.css";
@import "@dashtext/ui/styles/layout.css";
@import "@dashtext/ui/styles/tokyonight_day.css";
@import "@dashtext/ui/styles/tokyonight_moon.css";
```

**Step 3:** Commit

```bash
git add packages/web/src/app.html packages/web/src/app.css packages/web/src/app.d.ts packages/web/static/
git commit -m "feat(web): add app entry files to @dashtext/web"
```

---

## Phase 6: Cleanup and Verification

**Goal:** Remove old files, install dependencies, and verify everything works.

### Task 6.1: Remove old source directories

**Files:**
- Remove: `src/`
- Remove: `static/` (root level, now in packages)

**Step 1:** Remove old directories

```bash
rm -rf src/
rm -rf static/
```

**Step 2:** Commit

```bash
git rm -rf src/ static/
git commit -m "chore: remove old source directories after monorepo migration"
```

---

### Task 6.2: Remove old config files

**Files:**
- Remove: `svelte.config.js` (root level)
- Remove: `vite.config.js` (root level)
- Remove: `components.json` (or move to packages/ui if needed)

**Step 1:** Remove old config files

```bash
rm svelte.config.js vite.config.js components.json
```

**Step 2:** Commit

```bash
git rm svelte.config.js vite.config.js components.json
git commit -m "chore: remove old root config files"
```

---

### Task 6.3: Update documentation files

**Files:**
- Modify: `AGENTS.md`
- Modify: `CLAUDE.md`
- Modify: `README.md`

**Step 1:** Update AGENTS.md with new structure

Update the development commands section and architecture description to reflect the monorepo structure.

**Step 2:** Commit

```bash
git add AGENTS.md CLAUDE.md README.md
git commit -m "docs: update documentation for monorepo structure"
```

---

### Task 6.4: Install dependencies and verify builds

**Step 1:** Remove old bun.lock and node_modules

```bash
rm bun.lock
rm -rf node_modules
```

**Step 2:** Install fresh dependencies

```bash
bun install
```

**Step 3:** Run type checking

```bash
bun run check
```

**Step 4:** Build desktop app

```bash
bun run build:desktop
```

**Step 5:** Build web app

```bash
bun run build:web
```

**Step 6:** Commit lock file

```bash
git add bun.lock
git commit -m "chore: regenerate bun.lock for monorepo"
```

---

### Task 6.5: Test development servers

**Step 1:** Test desktop dev server

```bash
bun run dev:desktop
```

Verify the app loads at http://localhost:1420

**Step 2:** Test web dev server

```bash
bun run dev:web
```

Verify the app loads at http://localhost:5173 (or whatever port Vite assigns)

**Step 3:** Test Tauri dev

```bash
bun run tauri dev
```

Verify the Tauri window opens and the app works.

---

## Summary of Import Changes

When migrating code, use this reference for import updates:

| Old Import | New Import (Desktop) | New Import (Web) |
|------------|---------------------|------------------|
| `$lib/components/ui/button` | `@dashtext/ui/button` | `@dashtext/ui/button` |
| `$lib/components/ui/sidebar` | `@dashtext/ui/sidebar` | `@dashtext/ui/sidebar` |
| `$lib/components/editor` | `@dashtext/lib/editor` | `@dashtext/lib/editor` |
| `$lib/stores/drafts.svelte` | `@dashtext/lib/stores` | `@dashtext/lib/stores` |
| `$lib/api/types` | `@dashtext/lib/api` | `@dashtext/lib/api` |
| `$lib/api` | `$lib/api` (local) | `$lib/api` (local) |
| `$lib/utils` | `@dashtext/ui/utils` | `@dashtext/ui/utils` |
| `$lib/stores/updater.svelte` | `$lib/stores/updater.svelte` (local) | N/A |
| `$lib/components/updater` | `$lib/components/updater` (local) | N/A |
| `$lib/platform` | N/A (not needed) | N/A (not needed) |

---

## Execution Notes

This plan is designed for subagent execution. Each phase is independent after Phase 1 completes. Phases 2 and 3 can potentially run in parallel. Phase 4 and 5 depend on Phases 2 and 3 being complete.

**Recommended execution order:**
1. Phase 1 (Foundation) - Must be first
2. Phase 2 (UI) and Phase 3 (Lib) - Can run in parallel
3. Phase 4 (Desktop) - After 2 & 3
4. Phase 5 (Web) - After 2 & 3, can run parallel with Phase 4
5. Phase 6 (Cleanup) - Must be last
