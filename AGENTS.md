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

## Svelte/SvelteKit 5 

### Project Structure

**Co-locate by default**: SvelteKit ignores files without `+` prefix in route directories. Place components and utilities next to the routes that use them. Only move to `$lib` when code is shared across multiple routes.

**$lib/server is mandatory** for server-only code (database, API keys, private env vars).

### MCP Server

You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

#### Available MCP Tools:

##### 1. list-sections

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

##### 2. get-documentation

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

##### 3. svelte-autofixer

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

##### 4. playground-link

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.
