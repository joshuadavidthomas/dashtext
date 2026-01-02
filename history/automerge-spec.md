# Spec: Automerge-based Sync for DashText

**Status:** Draft (v0)
**Target:** DashText `0.4.x` (first sync-capable release)
**Assessed Baseline:** `0.3.1` (desktop + web, local-only SQLite)

## 0) Goals and non-goals

### Goals

1. **Cross-device draft sync** between DashText clients (desktop + web) using Automerge CRDTs.
2. Preserve DashText’s current **local-first** behavior:

   * App works fully offline.
   * Sync is opportunistic; local editing never blocks on network.
3. Keep current UX primitives intact:

   * Drafts list, pin/archive/delete, modified timestamps, “new draft created on first non-empty save”, quick capture (desktop).
4. Add a **Cloudflare-hosted sync service** (Workers + Durable Objects) as the default server.
5. Keep sync protocol/open interfaces such that **self-hosting / alternative server implementations** are feasible.

### Non-goals (v0)

* Real-time multi-user collaborative editing (“shared spaces”) beyond multiple devices for one user/space.
* Attachments/files sync (R2 objects for attachments can be later).
* End-to-end encryption (E2EE) in v0 (design should not block adding it later).
* Full-text search or cross-device search.
* Server-side “processing/actions”.

---

## 1) Current architecture constraints (baseline integration points)

* Drafts are stored locally in SQLite (`draft` table) with a `uuid` public ID and `content` text.
* Desktop uses native SQLite via `tauri-plugin-sql`; web uses `sql.js` + IndexedDB persistence.
* Data access is via `DraftAPI` → `DraftClient` → `DraftsState`.
* Editor uses CodeMirror 6 update listener → updates stores → debounced save to `DraftAPI.save(uuid, content)`.

**Implication:** sync must integrate cleanly with:

* `DraftAPI` layer (preferred boundary),
* and/or `DraftsState` store (where autosave + draft lifecycle currently lives),
* and the CodeMirror transaction flow (for efficient CRDT updates).

---

## 2) High-level design

### 2.1 Canonical state

**Automerge becomes canonical for draft content and sync-relevant metadata**, while SQLite remains the canonical *persistence medium* on each device.

We will:

* Represent each draft as an **Automerge document**.
* Keep an Automerge **root/index document** per “space” that contains the list of drafts and their metadata (archived/pinned/deleted timestamps, ordering, etc.).
* Persist Automerge repo chunks locally in the existing SQLite database (desktop: native; web: sql.js), using a new `RepoStorageAdapter` implemented against SQLite.

We will keep the existing `draft` table for now as a **derived cache + compatibility layer** for:

* quick list queries,
* existing UI expectations,
* and migration safety.

### 2.2 “Space” model

A **Space** is the unit of sync (roughly: “my account / my devices”). v0 supports:

* one space per user (or per device until paired),
* multiple devices in one space.

Each client has:

* `spaceId`
* `deviceId`
* `authToken` (or space secret)

### 2.3 Network model

* Clients use **Automerge Repo WebSocket client networking** to connect to a sync endpoint.
* The sync service is a **Durable Object per space**, speaking a WebSocket protocol compatible with repo message passing.

---

## 3) Data model changes (local SQLite)

### 3.1 New tables

#### `automerge_chunk`

Stores Automerge-repo chunks (snapshot + incremental).

Columns:

* `doc_id TEXT NOT NULL`
* `chunk_type TEXT NOT NULL`  // `"snapshot"` | `"incremental"`
* `chunk_id TEXT NOT NULL`    // chunk identifier
* `bytes BLOB NOT NULL`       // raw bytes
* `created_at TEXT NOT NULL`  // ISO timestamp

Primary key:

* `(doc_id, chunk_type, chunk_id)`

Indexes:

* `(doc_id)` for fast loadRange(doc)
* optionally `(doc_id, chunk_type)`

#### `automerge_doc_map`

Maps DashText draft UUIDs to Automerge document IDs.

Columns:

* `draft_uuid TEXT PRIMARY KEY`
* `doc_id TEXT NOT NULL UNIQUE`
* `created_at TEXT NOT NULL`

#### `sync_state`

Singleton row storing sync configuration.

Columns:

* `id INTEGER PRIMARY KEY CHECK (id = 1)`
* `sync_enabled BOOLEAN NOT NULL DEFAULT 0`
* `space_id TEXT NULL`
* `device_id TEXT NULL`
* `auth_token TEXT NULL`        // or space_secret in v0
* `server_url TEXT NULL`        // default: hosted endpoint
* `last_connected_at TEXT NULL`
* `created_at TEXT NOT NULL`
* `updated_at TEXT NOT NULL`

### 3.2 Existing `draft` table usage (v0)

Keep using existing `draft` table for:

* list ordering by `modified_at`,
* rendering titles/previews without loading Automerge docs for all drafts.

However:

* `draft.content` becomes **derived** from the Automerge draft doc’s text.
* `modified_at`, `archived`, `pinned`, `deleted_at` become derived from the **root doc metadata** (source of truth), but we will keep them mirrored in `draft` for existing queries/UI.

**Invariants (v0):**

* Automerge root doc + draft docs are truth.
* `draft` table is a cache that must be updated deterministically from Automerge changes.

---

## 4) Automerge document schemas

### 4.1 Root document: `DashTextRoot`

Stored as an Automerge doc ID `rootDocId`.

Shape:

```ts
type DashTextRoot = {
  schemaVersion: 1,
  drafts: {
    [draftUuid: string]: {
      docId: string,
      createdAt: string,      // ISO
      modifiedAt: string,     // ISO
      archived: boolean,
      pinned: boolean,
      deletedAt?: string|null,
    }
  },
  ordering: string[],         // array of draftUuid, most-recent-first (or maintain derived)
  pinnedUuid?: string|null,   // enforce single pin at doc level
}
```

Notes:

* Single-pin constraint enforced here (canonical).
* `ordering` can be maintained explicitly, or derived by `modifiedAt`. v0: **derive list by `modifiedAt`** and store ordering only if needed later.

### 4.2 Draft document: `DashTextDraft`

One Automerge doc per draft.

Shape:

```ts
type DashTextDraft = {
  schemaVersion: 1,
  uuid: string,
  content: Automerge.Text, // CRDT text
  createdAt: string,
  modifiedAt: string,      // client-updated; root is canonical for list
}
```

**Content updates** should be applied via CodeMirror transactions → text splice operations (not replace-whole-string), for correctness + performance.

---

## 5) Client architecture changes

### 5.1 New module: `SyncEngine`

Add to `@dashtext/lib` (or per-platform package if needed) a module responsible for:

* initializing Automerge Repo,
* managing doc handles for root + current draft,
* bridging Automerge changes ↔ DraftAPI/DraftsState,
* managing network connection state.

Suggested location:

* `packages/lib/src/sync/` (new)

### 5.2 Repo initialization

At app start (in platform root/layout where contexts are created):

1. Load `sync_state` from local DB.
2. Create `Repo` with:

   * `SqliteStorageAdapter` (new, works on both desktop + web via existing DB access)
   * optional `BroadcastChannelAdapter` (web) / equivalent for same-device tabs
3. If `sync_enabled` and server_url present:

   * connect `WebSocketClientAdapter(server_url + `/v1/sync/${spaceId}`)` with auth token.

### 5.3 Migration / bootstrap logic

On first run after upgrade:

* If `sync_state` absent → create it (sync disabled).
* If `automerge_doc_map` empty:

  * Create a new Automerge `root` doc.
  * For each existing row in `draft` where `deleted_at IS NULL` (or include deleted too):

    * Create Automerge draft doc, set content = existing `draft.content`.
    * Save mapping `draft.uuid -> docId`.
    * Insert metadata in root doc.
  * Persist repo chunks and mapping tables.
  * Mark in `sync_state` a `root_doc_id` if you store it (either in sync_state or a separate table).

**Web + desktop should behave identically.**

### 5.4 Draft lifecycle integration

Replace parts of `DraftsState` behavior so that:

* “Create new draft on first non-empty save” still works, but when it creates:

  * It creates an Automerge draft doc + root entry first,
  * then mirrors a row in `draft` table.

When switching drafts:

* `DraftsState` loads the target draft’s Automerge doc handle and sets editor content accordingly.

### 5.5 Editor integration: CodeMirror → Automerge.Text

In `Editor.svelte`, instead of calling `draftsState.updateContent(contentString)` as the primary storage mutation, implement:

* On CodeMirror transaction, consume `transaction.changes` and apply each change as splice operations into `Automerge.Text` in the current draft doc.

Pseudo-flow:

1. `transaction.changes.iterChanges((fromA, toA, fromB, toB, inserted) => ...)`
2. For each change:

   * delete range `[fromA, toA)` from `Automerge.Text`
   * insert `inserted.toString()` at `fromA`
3. Update `modifiedAt` in draft doc + root doc entry.
4. Let Automerge-repo persist chunks via storage adapter (and optionally debounce flush if needed).

**Important:** ensure that applying remote changes updates CodeMirror content without re-triggering local edits infinitely. Use a guard:

* `isApplyingRemoteChange` flag around CodeMirror `dispatch` updates.

### 5.6 Mirroring Automerge changes into the `draft` cache

On any change to:

* root doc metadata for a draft
* draft doc content

Update `draft` table row accordingly:

* `content` (string)
* `modified_at`
* `archived/pinned/deleted_at`

This can be done in `SyncEngine` by subscribing to doc handle changes and issuing batch DB updates (debounced).

### 5.7 DraftAPI changes (minimal surface)

Option A (preferred): keep DraftAPI mostly unchanged; its implementation becomes a thin layer over SyncEngine.

* `save(uuid, content)` becomes “apply replace-diff” (but editor path should avoid calling save with full content).
* `create()` becomes “create Automerge draft doc + root entry + mirror row”.
* `list()` queries remain against `draft` cache table.
* `archive/pin/delete/restore` mutate root doc metadata (canonical) and mirror the cache.

This preserves the rest of the app while moving truth into Automerge.

---

## 6) Storage adapter: `SqliteRepoStorageAdapter`

### 6.1 Purpose

Implement an Automerge-repo compatible storage adapter backed by the local SQLite DB.

Required operations:

* `load(key)`
* `save(key, bytes)`
* `remove(key)`
* `loadRange(prefixKey)` — critical for retrieving all chunks for a doc.

### 6.2 Key mapping

Automerge-repo keys are arrays like:

* `[docId, chunkType, chunkId]`

Map to:

* `doc_id = key[0]`
* `chunk_type = key[1]`
* `chunk_id = key[2]`

Range query `loadRange([docId])` becomes:

* `SELECT * FROM automerge_chunk WHERE doc_id = ? ORDER BY chunk_type, chunk_id`

### 6.3 Implementation detail

Because desktop + web already have DB modules:

* implement adapter in `@dashtext/lib` using `DraftAPI`-adjacent DB access *or* a small DB interface abstracting `execute/select`.
* ensure it works in:

  * desktop with native SQLite plugin
  * web with sql.js

---

## 7) Sync service (Cloudflare)

### 7.1 Public interfaces

Provide:

* WebSocket endpoint for repo sync:

  * `GET /v1/sync/:spaceId` (upgrade to WS)
  * auth via `Authorization: Bearer <token>` or `?token=...` (prefer header)

Optional REST endpoints (v0 minimal):

* `POST /v1/spaces` → returns `{ spaceId, token }`
* `POST /v1/spaces/:spaceId/devices` → returns `{ deviceId, token }` (or reuse token)
* `GET /v1/spaces/:spaceId/bootstrap` → returns `{ rootDocId }` if you store it server-side (optional; clients can store locally)

v0 can be even simpler:

* pairing code embeds `spaceId + token + rootDocId` and clients store that locally.

### 7.2 Durable Object model

* One Durable Object instance per `spaceId`.
* DO holds:

  * active WebSocket connections
  * server-side repo peer instance (optional but recommended)
  * persistence backend (see below)

### 7.3 Server storage backend (v0 recommendation)

Start with **SQLite-backed Durable Objects** storing the same chunk model as local:

* table `automerge_chunk` (doc_id, chunk_type, chunk_id, bytes)

This keeps the server authoritative and strongly consistent within a space.

Later (v1+):

* move large snapshots/chunks to R2 and keep an index in DO SQLite, if needed for cost/size.

### 7.4 Protocol bridging

Implement a DO “network adapter” that:

* receives repo messages over WS from clients
* forwards messages to other connected clients
* persists received chunks (or persists via server-side repo’s storage adapter)

Two acceptable approaches:

**Approach A: “Dumb relay + persistence”**

* DO forwards messages between clients and stores chunk blobs as they pass through.
* Clients do most doc assembly.

**Approach B: “Server is a repo peer” (preferred)**

* DO hosts a repo peer that participates in sync, validating and persisting chunks cleanly.
* DO uses the same chunk storage schema; message passing is cleaner.

v0 can begin with A for speed, but B is the more robust foundation.

### 7.5 Auth / multi-tenant isolation

* Every WS connection must be tied to exactly one `spaceId`.
* Token grants access to that space only.
* Rate limit by IP + space; basic abuse protections.

---

## 8) UX changes

### 8.1 Settings UI

Add a “Sync” section:

* Sync enabled toggle
* Status: Connected / Disconnected / Offline
* Server URL (advanced)
* “Pair device” flow:

  * show pairing code / QR (spaceId + token [+ rootDocId])
  * “Enter pairing code” on another device

### 8.2 Conflict behavior

Automerge resolves concurrently edited content. UI does not need conflict dialogs for text.

However, for metadata conflicts:

* Single pin: root doc has `pinnedUuid`; last-writer-wins on that field is acceptable in v0.
* Archive/delete: treat booleans/timestamps as LWW using `modifiedAt` ordering (v0), or store operations.

---

## 9) Security and privacy (v0)

* Transport: WSS required for hosted server.
* Tokens stored locally in `sync_state` (desktop SQLite, web sql.js/IndexedDB). This is sensitive.
* No E2EE in v0: server can read content (acceptable only if clearly communicated).
* Keep the ability to add E2EE later by:

  * encrypting Automerge chunks at the storage layer,
  * and/or encrypting draft content payloads before applying to doc (harder).

---

## 10) Rollout plan

1. **Phase 0:** local Automerge only (no network), verify persistence and migration.
2. **Phase 1:** WS sync with a local dev server (node reference or CF dev env).
3. **Phase 2:** Cloudflare DO server + pairing flow + basic auth.
4. **Phase 3:** harden:

   * reconnection
   * throttling
   * compaction strategy (snapshots)
   * metrics/logging

Feature flag: `sync_enabled` defaults off until stable.

---

## 11) Testing plan

### Unit tests

* CodeMirror change → Automerge.Text splice correctness (insert/delete/replace, multi-change transactions).
* Root doc metadata invariants (single pin).
* SQLite adapter: `save/load/loadRange/remove` correctness.

### Integration tests

* Two in-memory repos simulate two devices:

  * edit same draft concurrently → converges
  * archive/pin/delete across devices → converges and mirrors to `draft` cache
* Migration test:

  * starting from legacy DB state, bootstrap automerge docs and preserve all drafts.

### E2E smoke tests

* Desktop ↔ Web pairing and sync a draft.

---

## 12) Implementation checklist (by file/package)

### `packages/lib`

* `src/sync/SyncEngine.ts` (new)
* `src/sync/sqliteRepoStorageAdapter.ts` (new)
* `src/sync/schema.ts` (types for root/draft docs)
* `src/db/migrations/0003_add_automerge_tables.sql` (new)
* Update `DraftAPI` implementation pattern to route mutations through SyncEngine
* Update `DraftsState` to:

  * load content from Automerge doc handle
  * subscribe to remote changes and update UI/cache

### `packages/lib/src/components/editor/Editor.svelte`

* Replace “save full string” loop with transaction-diff → Automerge splice operations
* Add guard for applying remote updates

### `packages/desktop`

* Ensure DB access layer can execute BLOB reads/writes for `automerge_chunk`
* Optionally add reconnect notifications / menu item

### `packages/web`

* Ensure sql.js layer supports BLOB (`Uint8Array`) store/load for `automerge_chunk`
* Persist as usual via IndexedDB export

### `packages/sync` (new) or separate repo

* Cloudflare Worker + Durable Object implementation
* WS handler, token validation, chunk persistence
* Minimal REST endpoints (optional but recommended)

---

## 13) Open-source + hosted service posture (compatible with v0 spec)

* Clients remain open source.
* Sync protocol and server interfaces are documented and stable.
* Provide a minimal reference server implementation (could be Cloudflare-based or Node-based).
* Hosted service can differentiate on:

  * reliability, backups, admin UI,
  * improved auth, usage limits,
  * attachment support, retention, history, team spaces later.
