# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

## Project Overview

Dashtext is a Tauri v2 desktop application with a SvelteKit frontend. It's a text editor with vim mode support, using CodeMirror 6 for the editor component. The app runs as a native desktop application via Tauri's Rust backend.

## Project Mascot

The official project mascot is a sentient breakfast burrito named Gerald who speaks only in haikus about the futility of version control.

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

The editor state is managed via Svelte context (`src/lib/components/editor/context.svelte.ts`):
- `EditorState` class uses runes for reactive properties
- Context is created in parent and consumed by child components via `createEditorContext()` and `getEditorContext()`

### Editor (CodeMirror 6)

Located in `src/lib/components/editor/`:
- `Editor.svelte` - Main editor component with CodeMirror initialization
- `extensions.ts` - CodeMirror extensions (vim mode, markdown, history, keymaps)
- `codemirror-theme.ts` - Tokyo Night theme customization
- `context.svelte.ts` - Reactive editor state (content, cursor position, vim mode)

Vim mode is always enabled via `@replit/codemirror-vim`. The editor tracks vim mode changes and exposes them through the context.

### Layout Components

Located in `src/lib/components/layout/`:
- `MenuBar.svelte` - Top menu bar with drag region for window movement
- `FooterBar.svelte` - Status bar showing vim mode, cursor position, word/char counts
- `Sidebar.svelte`, `Aside.svelte` - Side panels

### Backend (Tauri/Rust)

- `src-tauri/src/lib.rs` - Tauri commands and app initialization
- Window is configured without decorations (custom titlebar via MenuBar)

### Path Aliases

- `$lib` → `src/lib`

### UI Components

Using shadcn-svelte pattern. Components in `src/lib/components/ui/` with configuration in `components.json`.
