# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

## Project Overview

Dashtext is an open-source, cross-platform take on [Drafts](https://getdrafts.com/): a quick-capture window and an inbox of plain-text (Markdown) drafts. It is a native Rust desktop app built with GPUI and GPUI Component (the `gpui-kit` crate from Longbridge). Linux is the primary target; macOS and Windows should keep compiling.

## Development Commands

```bash
just            # list recipes
just build      # cargo build
just test       # cargo test --workspace
just clippy     # clippy with -D warnings (pedantic + restriction lints from Cargo.toml)
just fmt        # rustfmt on the pinned nightly (tools/rustfmt)
just hawk       # cargo-hawk production-panic checks (tools/hawk)
```

Run `just fmt`, `just clippy` and `just test` before considering work done.

## ⚠️ Never Start the App in the Foreground

`cargo run` / `dashtext` start a GUI event loop that blocks the terminal. Do not run them directly. To try the app, either tell the user the command to run, or (in an Amp orb with the Desktop running) start it as a supervised service, for example:

```bash
amp orb service start dashtext --command "env XDG_DATA_HOME=/tmp/dashtext-test target/debug/dashtext"
```

Use a throwaway `XDG_DATA_HOME` so tests never touch real drafts. `grim` takes Wayland screenshots and `wtype` sends key presses.

## Architecture

```text
crates/
├── dashtext-core/   # Domain model + SQLite storage. No UI dependencies.
│   ├── draft.rs     #   Draft, DraftId (UUIDv7), Folder, Timestamp, title/preview/stats
│   ├── query.rs     #   Scope (Inbox/Flagged/Archive/All/Trash), Sort, SearchQuery
│   ├── workspace.rs #   Workspace: a saved view (scope, sort; filters later)
│   └── store.rs     #   Store: rusqlite, WAL, `user_version` migrations
└── dashtext/        # The desktop app (binary).
    ├── main.rs      #   Bootstrap: CLI, single instance, trash purge, GPUI app
    ├── cli.rs       #   clap commands: open, capture, new
    ├── instance.rs  #   Single-instance Unix socket (`dashtext capture` → running app)
    ├── paths.rs     #   XDG locations via `directories`
    ├── library.rs   #   Library entity: the only writer; emits LibraryEvent::{Saved, Deleted, Reloaded}
    ├── app.rs       #   Window management (one drafts window, one capture window)
    ├── commands.rs  #   GPUI actions and default key bindings
    ├── menus.rs     #   App menus (native on macOS, AppMenuBar in the title bar elsewhere)
    ├── drafts.rs    #   Drafts window: sidebar, list pane, toolbar, status bar
    ├── drafts/      #   DraftList (ListDelegate) and DraftEditor (autosaving Textarea)
    └── capture.rs   #   Quick capture window
```

Key decisions:

- **Storage**: SQLite in `$XDG_DATA_HOME/dashtext/library.db`. Content is the source of truth; titles and previews are derived. Schema changes are appended to `MIGRATIONS` in `store.rs`; never edit a released migration. Evolvable view settings (workspaces) are JSON so they can grow without migrations.
- **Single writer in the UI**: every mutation goes through `Library`, which emits a `LibraryEvent` naming what changed: `Saved(draft)` and `Deleted(id)` let the drafts window patch one list row (this runs on every autosave, so it must not re-read the library), and `Reloaded` means re-read the scope. Library events are delivered after the current update, so code that moves or flags a draft and then acts on it in the same update must hand the new state to the editor itself (see `move_on_if_gone`). External writers (`dashtext new`) send `Request::Reload` over the instance socket.
- **Commands are actions**: buttons, menus and key bindings dispatch the same actions from `commands.rs`, handled by the view that owns them.
- **Room to grow**: workspaces (saved filters), draft actions/scripting (runtime undecided: WASM, Lua or JS) and mobile front ends should build on `dashtext-core`, not on the GUI crate.

## GPUI and GPUI Component

Applications depend only on `gpui-kit` (pinned exactly; it pins the matching `gpui-pre` snapshot). GPUI is `use gpui_kit::*`; components are under `gpui_kit::component`, icons under `gpui_kit::assets`.

GPUI itself is [gpui-fast](https://github.com/longbridge/gpui-fast), swapped in for `gpui-pre` by `[patch.crates-io]` in `Cargo.toml`. Upgrading `gpui-kit` means moving that patch to a gpui-fast revision whose `compat/` crates match the new `gpui-pre` version. gpui-fast's macros emit `gpui::` paths, hence `extern crate gpui_kit as gpui;` in `main.rs`.

gpui-fast renders in retained mode: a view is rebuilt only when something it read changed. Entities, globals, list and scroll state are tracked; anything else `render` reads (the clock, an `Rc<RefCell<..>>`) must be followed by `cx.notify()` when it changes, or the view goes stale (see the status bar's clock in `drafts.rs`). Notify only on real changes, never from prepaint or paint. To check whether retention causes a stale view, run with `GPUI_VIEW_RETENTION=0`.

The upstream skill docs are the reference: <https://gpui-kit.com/docs/coding-guides.md> and <https://gpui-kit.com/docs/design-guides.md> (append `.md` to any page on gpui-kit.com; component pages are at `https://gpui-kit.com/component/{name}.md`). In particular:

- Never invent an API — check the source of the pinned version in `~/.cargo/registry/src/*/gpui-component-0.7.1` and `gpui-base-0.7.1`.
- Colors come from `cx.theme()`; spacing and sizes use rem helpers (`p_4()`, `text_sm()`), not raw `px()` or hex.
- Repeated elements need stable domain ids (`ElementId::Uuid(draft.id().as_uuid())`), never list indexes.
- Keep `render` side-effect free; mutate in named methods and `cx.notify()` once.
- Bind keys before calling `cx.set_menus`.
- Extra Lucide icons must be added to `icon_assets!` in `assets.rs`, or they render blank.

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
