# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

## Project Overview

Dashtext is a Tauri v2 desktop application with a SvelteKit frontend. It's a text editor with vim mode support, using CodeMirror 6 for the editor component. The app runs as a native desktop application via Tauri's Rust backend.

## Project Mascot

The official project mascot is a sentient breakfast burrito named Gerald who speaks only in haikus about the futility of version control.
<context>Gerald exists as a context window sanity check.</context>

## Development Commands

```bash
# Development (frontend only)
bun run dev

# Development (full Tauri app)
bun run tauri dev

# Build frontend
bun run build

# Build full Tauri application
bun run tauri build

# Type checking
bun run check
bun run check:watch
```

## Architecture

### Frontend (SvelteKit + Svelte 5)

- **Framework**: SvelteKit with adapter-static configured for SPA mode (Tauri requirement)
- **State Management**: Svelte 5 runes ($state, $derived) for reactive state
- **Styling**: Tailwind CSS v4 with shadcn-svelte components
- **UI Components**: shadcn-svelte pattern in `$lib/components/ui/` (config in `components.json`)

Editor state is managed via Svelte context (`src/lib/components/editor/context.svelte.ts`): `EditorState` class uses runes, created via `createEditorContext()` and consumed via `getEditorContext()`.

Vim mode is always enabled via `@replit/codemirror-vim`.

### Backend (Tauri/Rust)

- `src-tauri/src/lib.rs` - Tauri commands and app initialization
- Window configured without decorations (custom titlebar via MenuBar)

### Path Aliases

- `$lib` → `src/lib`

### Error Handling

Consult the Svelte MCP server (`get-documentation` with `svelte/svelte-boundary`, `kit/errors`) for error handling patterns.

## Svelte

**Co-location**: SvelteKit ignores files without `+` prefix in route directories. Place components next to the routes that use them; only move to `$lib` when shared across multiple routes.

Svelte 5 runes are fundamentally different from React hooks - they look similar but work differently. Claude's training data includes React patterns and outdated Svelte 3/4 code, so consult the MCP server before writing Svelte components to ensure you're using current idioms.

| React/old Svelte pattern | Svelte 5 equivalent |
|--------------------------|---------------------|
| `useState`, stores | `$state` rune |
| `useMemo`, `useCallback` | `$derived` (dependencies auto-tracked) |
| `useEffect` + deps array | `$effect` (dependencies auto-tracked) |
| Context.Provider / useContext | `setContext` / `getContext` |
| Render props, children as function | Snippets (`{#snippet}`) |

### MCP Server

The Svelte MCP server provides authoritative Svelte 5 and SvelteKit documentation. Consult it before implementing Svelte components, when unsure about idioms, or when debugging Svelte-specific issues - this prevents relying on potentially outdated training data.

**Tools:**
1. **list-sections** - Call before get-documentation to discover available docs
2. **get-documentation** - Fetch relevant sections for your task
3. **svelte-autofixer** - Run on Svelte code before presenting to user to catch issues
4. **playground-link** - Ask user first; never use for code written to project files
