# DashText Codebase Architecture Assessment

*Assessment Date: 2026-01-01*
*Version Assessed: 0.3.1*

---

## 1) Executive Summary

**DashText** is a desktop application for quick text capture, inspired by [Drafts](https://getdrafts.com/) for iOS/macOS. It provides vim-style editing via CodeMirror 6 and stores drafts locally in SQLite. The app is built as a Tauri v2 application with a SvelteKit frontend using Svelte 5 runes for state management. A web version exists that runs entirely client-side using sql.js (SQLite compiled to WebAssembly) with IndexedDB for persistence.

The application currently targets **Linux desktop** as the primary platform (other platforms are untested). A **web version** is deployable to Cloudflare Workers. The app is distributed as .deb, .rpm, .AppImage, and .tar.gz packages via GitHub Releases, with built-in self-update capability. There is no cloud sync, user authentication, or multi-device support at present — drafts are local-only.

---

## 2) Repository Layout & Packages

### 2.1 High-Level Folder Tree

```
dashtext/
├── packages/
│   ├── desktop/          # Tauri v2 desktop application
│   │   ├── src/          # SvelteKit frontend
│   │   ├── src-tauri/    # Rust backend (Tauri)
│   │   └── static/       # Static assets
│   ├── web/              # Web version (SPA)
│   │   ├── src/          # SvelteKit frontend
│   │   └── static/       # Static assets (includes sql.js WASM)
│   └── lib/              # Shared library (@dashtext/lib)
│       └── src/
│           ├── api/         # Draft API abstractions
│           ├── components/  # Shared UI components
│           ├── db/          # Schema & migrations
│           ├── hooks/       # Svelte hooks
│           ├── platform/    # Platform abstraction layer
│           ├── stores/      # State management (Svelte 5 runes)
│           └── styles/      # Shared CSS
├── .github/workflows/    # CI/CD (build.yml, release.yml)
├── assets/               # Documentation images
├── history/              # AI planning documents
└── .beads/               # Issue tracking (bd/beads)
```

### 2.2 Workspaces & Monorepo Setup

- **Package Manager**: Bun with workspaces
- **Root `package.json`**: Defines workspace packages and version catalog
- **Workspace Packages**:
  - `@dashtext/lib` (v0.0.1) — Shared library with components, stores, database schema
  - `@dashtext/desktop` (v0.3.1) — Tauri desktop application
  - `@dashtext/web` (v0.3.1) — Cloudflare-deployed web SPA

### 2.3 Shared Core vs Platform-Specific

| Package | Type | Purpose |
|---------|------|---------|
| `@dashtext/lib` | Shared Core | Editor, stores, components, DB schema, platform abstraction |
| `@dashtext/desktop` | Platform App | Tauri integration, window management, hotkeys, updater |
| `@dashtext/web` | Platform App | sql.js integration, IndexedDB persistence, Cloudflare deployment |

**Evidence Pointers**:
- Root package.json: `/home/user/dashtext/package.json:6-48` (workspaces config)
- Lib exports: `/home/user/dashtext/packages/lib/package.json:11-95` (module exports)

---

## 3) Runtime Targets & Build/Deploy

### 3.1 Desktop (Tauri v2)

| Aspect | Value |
|--------|-------|
| Framework | Tauri v2.9+ with tauri-plugin-sql (SQLite) |
| Rust Edition | 2024 |
| Window Config | Decorationless, custom titlebar via MenuBar |
| Bundle Formats | .deb, .rpm, .AppImage, .tar.gz |
| Signing | GPG (AppImage, RPM), Minisign (tar.gz) |
| Update Mechanism | Custom self-update via `latest.json` manifest |

**Tauri Plugins Used**:
- `tauri-plugin-sql` (SQLite with preload)
- `tauri-plugin-global-shortcut` (hotkey registration)
- `tauri-plugin-opener` (URL opening)
- `tauri-plugin-log` (logging)

**Update Mechanism Details**:
- Fetches `latest.json` from GitHub Releases
- Downloads tarball, verifies Minisign signature and SHA256
- Atomic binary swap with rollback capability
- Frontend emits `update-progress` events during download

**Evidence Pointers**:
- Tauri config: `packages/desktop/src-tauri/tauri.conf.json`
- Rust Cargo.toml: `packages/desktop/src-tauri/Cargo.toml`
- Updater implementation: `packages/desktop/src-tauri/src/updater.rs`

### 3.2 Web (Cloudflare Workers)

| Aspect | Value |
|--------|-------|
| Adapter | `@sveltejs/adapter-cloudflare` |
| Runtime | Cloudflare Workers |
| Database | sql.js (WASM SQLite) with IndexedDB persistence |
| SSR | Disabled (SPA mode) |
| Deploy Command | `wrangler deploy` |

**Wrangler Configuration**:
- Compatibility date: 2024-12-01
- Node.js compatibility enabled
- SPA mode for unmatched routes
- D1 database commented out (planned for sync)

**Evidence Pointers**:
- Wrangler config: `packages/web/wrangler.jsonc`
- SvelteKit config: `packages/web/svelte.config.js`

### 3.3 CI/CD (GitHub Actions)

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `build.yml` | Push to main, PRs | Build and test Linux packages |
| `release.yml` | GitHub Release published | Sign, build, upload release assets |
| `zizmor.yml` | - | Security scanning |

**Build Pipeline**:
1. Install Linux dependencies (webkit2gtk, libsoup3, etc.)
2. Setup Bun and Rust toolchains
3. Build frontend: `bun run build`
4. Build Tauri app: `bun run build:desktop`
5. Create tarball, sign with Minisign, generate checksums
6. Upload artifacts (deb, rpm, AppImage, tar.gz)

**Release Pipeline**:
1. Call build workflow with `sign: true`
2. Generate `latest.json` for updater
3. Upload all assets to GitHub Release

**Evidence Pointers**:
- Build workflow: `.github/workflows/build.yml`
- Release workflow: `.github/workflows/release.yml`

---

## 4) Data Model & Persistence

### 4.1 Storage Backends

| Platform | Storage | Location |
|----------|---------|----------|
| Desktop | Native SQLite via Tauri plugin | `sqlite:dashtext.db` (app data dir) |
| Web | sql.js (WASM) + IndexedDB | `dashtext-sqlite` IndexedDB store |

### 4.2 Schema Definitions

**Tables**:

1. **`draft`** — Primary draft storage
   - `id` (integer, PK, auto-increment)
   - `uuid` (text, unique, indexed) — Public identifier
   - `content` (text, default '')
   - `created_at` (text, ISO timestamp)
   - `modified_at` (text, ISO timestamp, indexed)
   - `deleted_at` (text, nullable, indexed) — Soft delete
   - `archived` (boolean, indexed)
   - `pinned` (boolean, indexed) — Single-pin constraint enforced in code

2. **`settings`** — Application settings
   - `id` (integer, PK, constrained to 1) — Singleton pattern
   - `capture_shortcut` (text, nullable)
   - `created_at` (text)
   - `updated_at` (text)

### 4.3 Migrations

Migrations are defined in `packages/lib/src/db/migrations/`:

| Version | File | Description |
|---------|------|-------------|
| 1 | `0000_clean_blockbuster.sql` | Create draft table with basic fields |
| 2 | `0001_gigantic_sharon_ventura.sql` | Add uuid, deleted_at, archived, pinned fields |
| 3 | `0002_add_settings_table.sql` | Add settings table |

**Migration Bundling**:
- Desktop: Included via Rust `include_str!` in `db.rs`
- Web: Bundled via Vite `?raw` imports in TypeScript

### 4.4 Data Access Layer

**ORM**: Drizzle ORM with `sqlite-proxy` adapter

**Pattern**:
```
Schema (packages/lib/src/db/schema.ts)
    ↓
DraftAPI interface (packages/lib/src/api/types.ts)
    ↓
Platform-specific backend:
  - Desktop: packages/desktop/src/lib/api/backend.ts
  - Web: packages/web/src/lib/api/backend.ts
    ↓
DraftClient wrapper (packages/lib/src/api/client.ts)
    ↓
DraftsState store (packages/lib/src/stores/drafts.svelte.ts)
```

**DraftAPI Interface Methods**:
- `list()`, `create()`, `get(uuid)`, `save(uuid, content)`
- `archive(uuid)`, `unarchive(uuid)`, `pin(uuid)`, `unpin(uuid)`
- `delete(uuid)` (soft), `hardDelete(uuid)`, `restore(uuid)`

### 4.5 Draft Representation

The `Draft` class (`packages/lib/src/stores/drafts.svelte.ts:9-58`) uses Svelte 5 runes:
- Reactive properties: `content`, `modified_at`, `deleted_at`, `archived`, `pinned`
- Derived properties: `title` (first line), `previewLines` (next 3 non-empty lines), `formattedModifiedAt`, `isDeleted`, `isActive`

### 4.6 Web Persistence (IndexedDB)

Web uses a two-tier persistence strategy:
1. **In-memory**: sql.js SQLite database
2. **Persistent**: IndexedDB store (`dashtext-sqlite`) with `Uint8Array` export

**Auto-save Triggers**:
- Debounced (1000ms) after mutations via `triggerAutoSave()`
- On `visibilitychange` (tab hidden)
- On `beforeunload` (page close)

**Evidence Pointers**:
- Schema: `packages/lib/src/db/schema.ts`
- Desktop DB: `packages/desktop/src/lib/db/index.ts`
- Web DB: `packages/web/src/lib/db/index.ts`
- Web persistence: `packages/web/src/lib/db/persist.ts`

---

## 5) Application Architecture

### 5.1 Frontend State Management

**Pattern**: Svelte 5 Runes with Context API

| Store | Purpose | Location |
|-------|---------|----------|
| `DraftsState` | Draft list, current draft, autosave | `packages/lib/src/stores/drafts.svelte.ts` |
| `EditorState` | Editor content, cursor position, vim mode | `packages/lib/src/components/editor/context.svelte.ts` |
| `SettingsState` | Vim enabled, onboarding completed | `packages/lib/src/stores/settings.svelte.ts` |

**Context Flow**:
```
Root Layout (+layout.svelte)
    └─ setPlatformContext(platform)
    └─ createSettingsContext()
        │
        ↓
EditorLayout (EditorLayout.svelte)
    └─ createEditorContext()
    └─ createDraftsState(initialDrafts, api, platform)
        │
        ↓
Editor, Sidebar, StatusLine (consume contexts)
```

### 5.2 Editor Architecture

**Stack**: CodeMirror 6 + @replit/codemirror-vim

**Components**:
- `Editor.svelte` — Main editor, manages CodeMirror view lifecycle
- `VimModeIndicator.svelte` — Displays current vim mode
- `extensions.ts` — CodeMirror extensions (vim, markdown, history, theme)
- `codemirror-theme.ts` — Tokyo Night theme definition

**Data Flow**:
```
User types
    ↓
EditorView.updateListener
    ↓
editorState.setContent(content)  // Update EditorState
draftsState.updateContent(content)  // Trigger autosave
    ↓
debouncedSave (500ms)
    ↓
DraftAPI.save(uuid, content)
```

**Autosave Mechanism**:
- Debounced via `runed.useDebounce` (500ms delay)
- `flushPendingSave()` forces immediate save (called on navigation)
- New drafts created on first non-empty save, URL updated via `replaceUrl`

**Undo/Redo**:
- Handled by CodeMirror's `history()` extension
- No custom undo stack management

### 5.3 Separation of Concerns

| Layer | Responsibility | Example |
|-------|---------------|---------|
| UI Components | Rendering, user interaction | `Editor.svelte`, `DraftsSidebar.svelte` |
| State Stores | Reactive state, business logic | `DraftsState`, `EditorState` |
| API Layer | Data operations abstraction | `DraftAPI`, `DraftClient` |
| Platform Layer | Platform-specific capabilities | `PlatformCapabilities` interface |
| Backend | Storage implementation | Desktop: Tauri SQLite, Web: sql.js |

### 5.4 Key Modules/Services

| Module | Responsibility | Path |
|--------|---------------|------|
| EditorLayout | Main app shell with sidebar | `lib/components/editor-layout/` |
| DraftsSidebar | Draft list display | `lib/components/editor-layout/DraftsSidebar.svelte` |
| BaseStatusLine | Status bar (vim mode, word count, position) | `lib/components/editor-layout/BaseStatusLine.svelte` |
| BaseWinBar | Window controls (minimize, maximize, close) | `lib/components/editor-layout/BaseWinBar.svelte` |
| Settings | Settings dialog, onboarding, hotkey input | `lib/components/settings/` |
| Capture (desktop) | Quick capture window | `desktop/src/lib/components/capture/` |
| Updater (desktop) | Auto-update UI | `desktop/src/lib/components/updater/` |

### 5.5 Plugin/Action System

**Current State**: No plugin or action system exists. The README mentions "Draft actions/processing" as a "Someday/Maybe" feature. Issue tracking (beads) shows extensive planning for an actions system (Epic 3.x) including:
- Built-in actions (Copy, Open URL, Share, Append to file)
- Custom action definitions
- Action execution engine

**Evidence Pointers**:
- Editor: `packages/lib/src/components/editor/Editor.svelte`
- DraftsState: `packages/lib/src/stores/drafts.svelte.ts`
- EditorLayout: `packages/lib/src/components/editor-layout/EditorLayout.svelte`

---

## 6) Networking & Backend (Current State)

### 6.1 Server Components

**Current**: None. Both desktop and web versions are fully offline/local-only.

**Planned** (per beads issues):
- Cloudflare Worker with Durable Object for sync (Epic M1)
- POST /api/drafts endpoint for API ingestion (Epic 2.1)
- D1 database integration (commented in wrangler.jsonc)

### 6.2 Authentication

**Current**: None. No user accounts, no authentication.

**Planned** (per beads issues):
- Token-based API authentication (issue `dashtext-0sh`)
- JWT auth middleware for sync (Epic M1)

### 6.3 Remote Storage

**Current**: None. All data is local.

**Planned**:
- Cloudflare D1 database for sync
- Durable Objects for per-user sync coordination

### 6.4 Error Logging/Analytics/Telemetry

**Current**:
- `tauri-plugin-log` for desktop logging (Rust side)
- `console.error` for frontend errors
- No external analytics or telemetry

**Evidence Pointers**:
- Tauri plugins: `packages/desktop/src-tauri/src/lib.rs:28`
- Wrangler D1 config (commented): `packages/web/wrangler.jsonc:34-41`

---

## 7) Platform Abstractions

### 7.1 PlatformCapabilities Interface

Defined in `packages/lib/src/platform/types.ts`:

```typescript
interface PlatformCapabilities {
  platform: 'desktop' | 'web';
  onFocusChange(callback: () => void): () => void;
  window: { minimize, maximize, close } | null;
  quickCapture: { open() } | null;
  settings: { open() } | null;
  draftsAPI: DraftAPI;
  refreshDrafts(): Promise<Draft[]>;
  replaceUrl(url: string): void;
  navigateTo(url: string): Promise<void>;
}
```

### 7.2 Platform Implementations

| Capability | Desktop | Web |
|------------|---------|-----|
| Window controls | Tauri window APIs | `null` |
| Quick capture | Opens new Tauri WebviewWindow | `null` |
| Settings | Opens settings window | `null` (uses dialog) |
| Focus change | `getCurrentWindow().onFocusChanged()` | `visibilitychange` event |
| Navigation | SvelteKit `goto`/`replaceState` | Same |
| Drafts API | SQLite via Tauri plugin | sql.js + IndexedDB |

### 7.3 Desktop vs Web Differences

| Feature | Desktop | Web |
|---------|---------|-----|
| Database | Native SQLite | sql.js WASM |
| Persistence | Automatic (SQLite file) | Manual (IndexedDB export) |
| Window chrome | Custom (decorationless) | Browser chrome |
| Global hotkey | Yes (evdev/Global Shortcut) | No |
| Quick capture | Yes (separate window) | No |
| Auto-update | Yes | N/A |
| File access | Yes (planned actions) | No |

### 7.4 Tauri API Surface

**Commands Exposed to Frontend**:
- `register_capture_shortcut(shortcut: string)`
- `unregister_capture_shortcut()`
- `check_update()`, `can_auto_update()`, `download_and_install_update()`, `restart_app()`, `get_current_version()`

**Events Emitted**:
- `hotkey:capture` — Global hotkey triggered
- `update-progress` — Download progress during update

**Capabilities** (from `default.json`):
- Core window operations (close, focus, show, center)
- WebView window creation
- Global shortcut registration
- SQL execution
- URL opening

**Evidence Pointers**:
- Platform interface: `packages/lib/src/platform/types.ts`
- Desktop platform: `packages/desktop/src/lib/platform.ts`
- Web platform: `packages/web/src/lib/platform.ts`
- Tauri capabilities: `packages/desktop/src-tauri/capabilities/default.json`

---

## 8) Security, Privacy, and Data Sensitivity

### 8.1 User Data Stored

| Data | Storage | Sensitivity |
|------|---------|-------------|
| Draft content | SQLite (local) | User content — potentially sensitive |
| Settings | SQLite + localStorage | Low sensitivity |
| Capture shortcut | SQLite | Low sensitivity |

### 8.2 Encryption

| Layer | Status |
|-------|--------|
| At Rest | **None** — SQLite database is unencrypted |
| In Transit | **N/A** — No network communication currently |
| Update verification | Minisign signatures + SHA256 checksums |

### 8.3 Threat-Sensitive Areas

1. **Local File Access** (desktop):
   - SQLite database readable by any process with user permissions
   - No encryption means content exposed if device compromised
   - Planned file append action would need path validation

2. **Update Mechanism**:
   - Downloads from hardcoded GitHub URL
   - Verifies Minisign signature with embedded public key
   - Atomic swap with rollback mitigates partial updates
   - **Risk**: Compromised signing key would enable malicious updates

3. **Global Hotkey** (Linux):
   - evdev backend requires `/dev/input/` access
   - Falls back to X11/Wayland global shortcut if evdev unavailable
   - **Risk**: evdev requires elevated permissions on some systems

4. **Web IndexedDB**:
   - Data persisted in browser's IndexedDB
   - Accessible to any JavaScript on the same origin
   - No encryption

### 8.4 CSP Configuration

- CSP is **disabled** (`"csp": null` in tauri.conf.json)
- Web version inherits Cloudflare's default headers

**Evidence Pointers**:
- Tauri security config: `packages/desktop/src-tauri/tauri.conf.json:14-16`
- Update verification: `packages/desktop/src-tauri/src/updater.rs:206-224`

---

## 9) Notable Constraints & Non-Obvious Behavior

### 9.1 Performance-Sensitive Paths

1. **Editor Input** — CodeMirror handles all keystroke processing; debounced saves prevent DB thrashing
2. **Draft List Rendering** — Derived properties (title, preview) computed per draft; list sorted by `modified_at`
3. **App Startup** (desktop) — Window hidden until Svelte mount, then shown via `showWindowWhenReady()`
4. **Database Initialization** (web) — sql.js WASM must load before any DB operations

### 9.2 Known Limitations

1. **Single Pin Constraint** — Only one draft can be pinned; implemented in API layer, not DB constraint
2. **No Search** — No full-text search or filtering of drafts
3. **No Sync** — Drafts are local-only; sync infrastructure extensively planned but not implemented
4. **Linux Focus** — macOS/Windows untested (per README)
5. **No Undo Across Sessions** — CodeMirror history is per-session only

### 9.3 Non-Obvious Behavior

1. **Draft Creation Timing** — Drafts aren't created until first non-empty save:
   - User navigates to `/drafts/new`
   - Types content
   - Debounced save triggers `DraftAPI.create()` + `DraftAPI.save()`
   - URL replaced to `/drafts/{uuid}`

2. **Vim Mode Toggle** — Changing vim setting recreates the entire CodeMirror editor (via Svelte `{#key}`):
   ```svelte
   {#key vimKey}
     <div use:initEditor ...></div>
   {/key}
   ```

3. **Focus Refresh** — On window/tab focus, drafts are re-fetched from database to catch external changes

4. **Web Auto-Save** — Debounced differently than desktop:
   - Mutations call `triggerAutoSave()` (1000ms debounce)
   - Also saves on `visibilitychange` and `beforeunload`

5. **Settings Singleton** — Settings table enforced to single row via SQL constraint: `CHECK (id = 1)`

### 9.4 Discovered from Issue Tracking (beads)

The codebase has extensive planning documents for future features:
- **Phase 5 — Sync**: Rev-based optimistic concurrency, Cloudflare Durable Objects
- **Epic 2.x — API Ingestion**: Token auth, POST /api/drafts, rate limiting
- **Epic 3.x — Actions**: Built-in actions, scripting support

**Evidence Pointers**:
- Draft creation flow: `packages/lib/src/stores/drafts.svelte.ts:122-147`
- Vim key recreation: `packages/lib/src/components/editor/Editor.svelte:174-176`
- Focus refresh: `packages/lib/src/components/editor-layout/EditorLayout.svelte:36-41`
- Beads issues: `.beads/issues.jsonl`

---

## 10) Appendix: Evidence Pointers

### 10.1 Major Claims with File References

| Section | Claim | Evidence |
|---------|-------|----------|
| 2 | Bun workspaces with catalog | `package.json:6-47` |
| 3 | Tauri v2 with plugins | `packages/desktop/src-tauri/Cargo.toml:34-39` |
| 3 | Self-update via latest.json | `packages/desktop/src-tauri/src/updater.rs:12-14` |
| 4 | Drizzle ORM schema | `packages/lib/src/db/schema.ts` |
| 4 | DraftAPI interface | `packages/lib/src/api/types.ts:18-30` |
| 5 | DraftsState with autosave | `packages/lib/src/stores/drafts.svelte.ts:66-147` |
| 5 | EditorState runes | `packages/lib/src/components/editor/context.svelte.ts:17-62` |
| 6 | No server components | Verified: no API routes, no server endpoints |
| 7 | PlatformCapabilities interface | `packages/lib/src/platform/types.ts:8-41` |
| 8 | CSP disabled | `packages/desktop/src-tauri/tauri.conf.json:15` |
| 8 | Minisign verification | `packages/desktop/src-tauri/src/updater.rs:206-224` |

### 10.2 Commands Run

```bash
# Directory structure
tree /home/user/dashtext -L 3 -I 'node_modules|target|.git|.svelte-kit' --dirsfirst

# File listings
ls -la /home/user/dashtext/packages/lib/src/
ls -la /home/user/dashtext/packages/desktop/src-tauri/src/
ls -la /home/user/dashtext/.github/workflows/

# Beads issues
cat /home/user/dashtext/.beads/issues.jsonl | head -30
```

### 10.3 Key Symbols Reference

| Symbol | Type | Location | Purpose |
|--------|------|----------|---------|
| `Draft` | Class | `lib/src/stores/drafts.svelte.ts:9` | Reactive draft model |
| `DraftsState` | Class | `lib/src/stores/drafts.svelte.ts:66` | Draft state management |
| `EditorState` | Class | `lib/src/components/editor/context.svelte.ts:17` | Editor state |
| `DraftAPI` | Interface | `lib/src/api/types.ts:18` | Storage operations |
| `PlatformCapabilities` | Interface | `lib/src/platform/types.ts:8` | Platform abstraction |
| `createExtensions` | Function | `lib/src/components/editor/extensions.ts:46` | CodeMirror setup |
| `tauriBackend` | Object | `desktop/src/lib/api/backend.ts:28` | Desktop DraftAPI impl |
| `webBackend` | Object | `web/src/lib/api/backend.ts:29` | Web DraftAPI impl |
| `SettingsState` | Struct (Rust) | `desktop/src-tauri/src/conf.rs:8` | Hotkey management |
| `HotkeyManager` | Trait (Rust) | `desktop/src-tauri/src/hotkey.rs:9` | Hotkey abstraction |

---

*End of Assessment*
