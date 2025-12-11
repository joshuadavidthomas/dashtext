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

## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Auto-syncs to JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**
```bash
bd ready --json
```

**Create new issues:**
```bash
bd create "Issue title" -t bug|feature|task -p 0-4 --json
bd create "Issue title" -p 1 --deps discovered-from:bd-123 --json
bd create "Subtask" --parent <epic-id> --json  # Hierarchical subtask (gets ID like epic-id.1)
```

**Claim and update:**
```bash
bd update bd-42 --status in_progress --json
bd update bd-42 --priority 1 --json
```

**Complete work:**
```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task**: `bd update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`
6. **Commit together**: Always commit the `.beads/issues.jsonl` file together with the code changes so issue state stays in sync with code state

### Auto-Sync

bd automatically syncs with git:
- Exports to `.beads/issues.jsonl` after changes (5s debounce)
- Imports from JSONL when newer (e.g., after `git pull`)
- No manual export/import needed!

### GitHub Copilot Integration

If using GitHub Copilot, also create `.github/copilot-instructions.md` for automatic instruction loading.
Run `bd onboard` to get the content, or see step 2 of the onboard instructions.

### Managing AI-Generated Planning Documents

AI assistants often create planning and design documents during development:
- PLAN.md, IMPLEMENTATION.md, ARCHITECTURE.md
- DESIGN.md, CODEBASE_SUMMARY.md, INTEGRATION_PLAN.md
- TESTING_GUIDE.md, TECHNICAL_DESIGN.md, and similar files

**Best Practice: Use a dedicated directory for these ephemeral files**

**Recommended approach:**
- Create a `history/` directory in the project root
- Store ALL AI-generated planning/design docs in `history/`
- Keep the repository root clean and focused on permanent project files
- Only access `history/` when explicitly asked to review past planning

**Example .gitignore entry (optional):**
```
# AI planning documents (ephemeral)
history/
```

**Benefits:**
- ✅ Clean repository root
- ✅ Clear separation between ephemeral and permanent documentation
- ✅ Easy to exclude from version control if desired
- ✅ Preserves planning history for archeological research
- ✅ Reduces noise when browsing the project

### CLI Help

Run `bd <command> --help` to see all available flags for any command.
For example: `bd create --help` shows `--parent`, `--deps`, `--assignee`, etc.

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ✅ Store AI planning docs in `history/` directory
- ✅ Run `bd <cmd> --help` to discover available flags
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems
- ❌ Do NOT clutter repo root with planning documents
