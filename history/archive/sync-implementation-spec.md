# Notes Sync Implementation Spec

Single-user today, multi-device now (desktop + web), mobile later. Offline-first on every client. Central sync service on Cloudflare **Durable Objects (DO)**. Web local store is **IndexedDB** (with a simple schema-version upgrader). Desktop local store is **SQLite** (you can use Drizzle + migrations there). No CRDTs in v1; conflicts are detected and preserved as "conflict copies."

---

## 1. Goals

### Must-have

* Works offline on every client (desktop + web).
* Edits on any device eventually sync everywhere.
* No silent data loss: avoid naïve last-write-wins overwrites.
* Simple, predictable operational model on Cloudflare.

### Nice-to-have

* Per-note history/versions (at least server-side first).
* Conflict-friendly UX (conflict copies now; merge UI later).

### Non-goals for v1

* Real-time collaborative editing (multi-user).
* Character-level merges / CRDTs.
* Server-side full-text search (especially if E2EE later).

---

## 2. System architecture

### Components

1. **Desktop Client (Tauri)**

* Local DB: SQLite
* Sync agent: background task that pushes/pulls

2. **Web Client (Svelte)**

* Local DB: IndexedDB
* Sync agent: background task that pushes/pulls

3. **Central Sync Service (Cloudflare Durable Object)**

* One DO instance per user (or per account).
* DO persists state using **SQLite storage** (Durable Objects SQL storage).
* DO serializes writes naturally (single-threaded per object), simplifying correctness.

### Data flow (high level)

* UI reads/writes local DB immediately.
* Client records changes in a local "outbox" (or dirty flags).
* Sync agent periodically calls server `/sync`.
* Server applies changes with optimistic concurrency checks and returns:

  * acks for pushed changes
  * any new changes since last sync
* Client applies server updates to local DB.

---

## 3. Identity and tenancy

### Identity

* Use a JWT-based provider (Cloudflare Access, Clerk, Auth0, etc.).
* Every request includes `Authorization: Bearer <jwt>`.
* Worker verifies JWT, extracts stable `user_id` (`sub`).

### DO routing

* DO ID derived from `user_id`:

  * `const id = env.SYNC_DO.idFromName(user_id)`
  * `const stub = env.SYNC_DO.get(id)`
* Worker forwards request to DO stub.

**Rule:** server never trusts client-supplied `user_id`.

---

## 4. Data model

### Canonical note fields (shared conceptually across all stores)

* `id: UUID`
* `title: string` (optional)
* `content: string`
* `deleted: boolean` (tombstone, never hard-delete immediately)
* `rev: integer` (server-controlled monotonic per note)
* `updated_at: integer` (server timestamp; UI only)

### Server-side additional fields

* `seq: integer` (server-controlled monotonic per user; for incremental pull)
* Optional history table for versions

---

## 5. Local storage specs

### 5.1 Desktop (SQLite)

Tables (suggested minimal):

* `notes(id PRIMARY KEY, title, content, deleted, rev, updated_at, dirty)`
* `outbox(id PRIMARY KEY, note_id, op, base_rev, title, content, created_at)`
* `meta(key PRIMARY KEY, value)` (stores `last_seq`, schema version)

**Migrations**

* Use Drizzle migrations here if you like.
* Schema version stored in `meta`.

### 5.2 Web (IndexedDB)

Database: `notes_app`
Object stores:

* `notes` (key: `id`)

  * fields: title/content/deleted/rev/updated_at/dirty
* `outbox` (key: auto-increment or UUID)

  * fields: note_id/op/base_rev/title/content/created_at
* `meta` (key: `key`)

  * fields: `last_seq`, `schema_version`

**Migration strategy**

* IndexedDB `onupgradeneeded` with incremental upgrades by schema version.
* Keep it simple: append fields, create new stores, backfill where needed.

---

## 6. Server DO storage schema

Inside the Durable Object, maintain SQLite tables:

### `notes`

* `id TEXT PRIMARY KEY`
* `title TEXT`
* `content TEXT`
* `deleted INTEGER NOT NULL DEFAULT 0`
* `rev INTEGER NOT NULL`
* `updated_at INTEGER NOT NULL`

### `changes`

* `seq INTEGER PRIMARY KEY` (monotonic, per user/DO)
* `note_id TEXT NOT NULL`
* `rev INTEGER NOT NULL`
* `op TEXT NOT NULL` (`upsert`/`delete`)
* `changed_at INTEGER NOT NULL`

(Optional) if you want "thin pull" responses without extra lookups, you can denormalize:

* store `title/content/deleted` in `changes` too. (Costs more storage; saves reads.)

### `note_versions` (optional, recommended early)

* `note_id TEXT NOT NULL`
* `rev INTEGER NOT NULL`
* `title TEXT`
* `content TEXT`
* `deleted INTEGER NOT NULL`
* `saved_at INTEGER NOT NULL`
* `PRIMARY KEY(note_id, rev)`

### `meta`

* `key TEXT PRIMARY KEY`
* `value TEXT`
* store `next_seq` (or compute with MAX(seq)+1, but storing is cheaper)

---

## 7. Sync protocol

### Request: `POST /sync`

```json
{
  "last_seq": 123,
  "changes": [
    {
      "client_change_id": "uuid-optional",
      "op": "upsert",
      "note_id": "uuid",
      "base_rev": 7,
      "title": "optional",
      "content": "string",
      "deleted": false
    }
  ],
  "limit": 500
}
```

### Response

```json
{
  "ack": [
    {
      "note_id": "uuid",
      "client_change_id": "uuid-optional",
      "status": "ok",
      "rev": 8,
      "updated_at": 1734300000
    },
    {
      "note_id": "uuid2",
      "status": "conflict",
      "server": {
        "id": "uuid2",
        "title": "...",
        "content": "...",
        "deleted": false,
        "rev": 9,
        "updated_at": 1734300100
      }
    }
  ],
  "updates": [
    {
      "op": "upsert",
      "note": {
        "id": "uuid3",
        "title": "...",
        "content": "...",
        "deleted": false,
        "rev": 4,
        "updated_at": 1734300200
      },
      "seq": 124
    }
  ],
  "new_last_seq": 130,
  "has_more": false
}
```

**Notes**

* `last_seq` is the client's checkpoint of "I've applied all changes up to seq X."
* `updates` are ordered by `seq`.
* `limit/has_more` supports pagination for first sync or large backlogs.

---

## 8. Server sync algorithm (Durable Object)

### Core invariants

* **rev is authoritative** and monotonically increases per note.
* **seq is authoritative** and monotonically increases per DO/user.
* Every accepted write produces exactly one change entry (seq increments).

### Apply pushed changes (pseudo)

For each incoming change:

1. Load current server note by `note_id` (or treat missing as `rev = 0`).
2. If `base_rev != current.rev` → return `conflict` with server note.
3. Else accept:

   * Save current version into `note_versions` (optional).
   * New `rev = current.rev + 1` (or `1` if create)
   * Write note row (`title/content/deleted/rev/updated_at=now`)
   * Append `changes(seq, note_id, rev, op, changed_at=now)` with `seq = next_seq++`
   * Return `ok` ack with new `rev`.

### Pull updates

* Query `changes WHERE seq > last_seq ORDER BY seq LIMIT limit`
* For each change:

  * fetch note from `notes` (or if denormalized, skip extra reads)
  * include in `updates`

Return `new_last_seq = max(seq returned)` (or original last_seq if none).

**Because DO is single-threaded, you can safely implement `next_seq` as "meta counter" without contention.**

---

## 9. Client sync algorithm

### Local outbox model

* Every local mutation produces an outbox entry:

  * `op` (`upsert`/`delete`)
  * `note_id`
  * `base_rev` (the last known server rev at the time of edit)
  * new values

### Sync loop

1. Read `last_seq` from meta.
2. Gather pending outbox entries (bounded batch, e.g. 100).
3. POST `/sync` with `{ last_seq, changes }`.
4. Process `ack`:

   * `ok`: update local note `rev`, `updated_at`, clear dirty; remove outbox entry.
   * `conflict`: do **conflict copy** flow (below); remove outbox entry (or keep in "needs attention" state).
5. Apply `updates` in increasing `seq`:

   * upsert/delete into local notes
   * set `dirty = false` for server-applied versions
6. Set local `last_seq = response.new_last_seq`.

### Conflict copy policy (v1)

When server says conflict:

* Create a new note locally:

  * `id = new uuid`
  * title = `"(Conflict) " + original title`
  * content = client's attempted content
  * `rev = 0` (local-only until first sync)
  * mark dirty/outbox as a create on next sync
* Replace local original note with server version (so device converges)
* Optionally show a banner "Conflict detected, saved a copy."

This ensures no data loss, no merge UI required.

---

## 10. Deletions and tombstones

* Delete is represented as `deleted = true` (tombstone).
* Tombstones sync like normal updates (rev increments).
* Hard-delete can be a later GC job (e.g. if deleted for >30 days).

---

## 11. History and versions

Minimum viable:

* Maintain `note_versions` on server for every accepted update (store prior version).

Expose later:

* "History" UI can fetch versions for a note:

  * `GET /notes/:id/versions?before_rev=...`
* For now, versions are primarily a safety net and future feature.

---

## 12. Optional: End-to-end encryption

You can add E2EE later without changing the sync mechanics by encrypting:

* `title` and `content` client-side
* server stores ciphertext strings
* metadata remains in clear (id, rev, seq, deleted, updated_at)

Decide early if you want this, because it affects:

* how you do search (local-only)
* recovery flows (key management)

---

## 13. API surface

Worker routes:

* `POST /sync` → forwards to DO
* (later) `GET /export`, `GET /versions`, `POST /import`

Durable Object:

* handles `/sync` internally
* maintains SQLite schema init on first request

---

## 14. Operational considerations

### Rate limits / batching

* Batch outbox changes (e.g. 100 at a time).
* Pagination on pull via `limit`.

### Idempotency

* Include `client_change_id` optionally.
* Server can store a small recent "seen ids" set to dedupe retries (optional v1).
* Alternatively: client retries are safe-ish if you only remove outbox on `ok` and treat conflicts deterministically.

### Observability

* Log sync sizes: pushed count, pulled count, conflicts count.
* Track latency and error rates per endpoint.
* Consider a per-client "sync status" indicator in UI.

---

## 15. Testing plan

### Correctness scenarios

* Offline edit on desktop → later sync to web.
* Offline edit on web → later sync to desktop.
* Concurrent edit same note on both while offline → conflict copy created.
* Delete on one device while other edits → conflict path (treat delete as a normal update; preserve both via conflict copy).

### Property tests (nice)

* Random sequence of operations across two simulated clients:

  * after eventual sync, both converge on same canonical note set
  * no edit is silently dropped (conflicts create extra notes)

---

## 16. Milestones

### M0: Data layer

* Desktop SQLite schema + outbox
* Web IndexedDB schema + upgrader + outbox

### M1: Central DO sync

* Worker auth middleware
* DO schema init + `/sync` implementation
* Minimal `/sync` protocol end-to-end

### M2: UX hardening

* Conflict copy UI message
* Sync status indicator
* Backoff/retry logic

### M3: History groundwork

* `note_versions` table + write-on-update
* (Optional) expose versions endpoint

---

## Appendix A: Minimal "shape contracts" (TypeScript-ish)

### Note

```ts
type Note = {
  id: string
  title: string | null
  content: string
  deleted: boolean
  rev: number
  updated_at: number
}
```

### OutboxChange

```ts
type OutboxChange = {
  client_change_id: string
  op: "upsert" | "delete"
  note_id: string
  base_rev: number
  title?: string | null
  content?: string
  deleted?: boolean
  created_at: number
}
```

---

## Decision Context

This spec represents a conscious architectural commitment to:

- **Design B** (rev-based optimistic concurrency) over Design A (LWW) or Design C (CRDT)
- **Hand-rolled** Cloudflare Durable Objects over sync engines (ElectricSQL, PowerSync) or CRDT stacks (Automerge, Yjs)
- **Conflict copies** as the UX pattern for conflicts (no silent overwrites, no character-level merges)

This gets DashText multi-device sync with:
- Minimal operational complexity (single DO per user, no distributed coordination)
- Clear mental model (rev-based conflict detection, explicit conflict resolution)
- Evolution path flexibility (can add CRDTs or migrate to sync engines later if needed)
