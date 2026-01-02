# Archived Specifications

Documents in this directory represent explored approaches that were not implemented.

## sync-implementation-spec.md (Dec 2025)

**Status:** Not implemented (archived Jan 2026)

**What it was:** Rev-based optimistic concurrency sync with outbox queue and conflict copies.

**Why archived:** Automerge CRDT approach chosen instead. While the outbox approach was simpler to understand, Automerge handles conflict resolution automatically and reduces implementation risk for sync's hardest problems.

**See instead:** ../automerge-spec.md (current implementation)
