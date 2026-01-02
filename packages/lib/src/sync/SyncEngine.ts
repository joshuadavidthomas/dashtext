/**
 * SyncEngine - Core Automerge integration for DashText
 *
 * Manages Automerge Repo initialization, document handles for root and drafts,
 * and provides reactive subscriptions for the UI layer.
 *
 * Phase 0: Local-only persistence (no network adapters)
 * Phase 2+: Add network adapters for cross-device sync
 */

import { Repo, type DocHandle, type DocumentId, type PeerId } from '@automerge/automerge-repo';
import type { StorageAdapterInterface, NetworkAdapterInterface } from '@automerge/automerge-repo';
import * as Automerge from '@automerge/automerge';
import type { DashTextRoot, DashTextDraft, DraftMetadata } from './types';
import type { DbExecutor, DocMapRow } from './storage/db-executor';

/**
 * Configuration for SyncEngine initialization
 */
export interface SyncEngineConfig {
  /** Storage adapter for persisting Automerge chunks */
  storage: StorageAdapterInterface;
  /** Database executor for doc mapping queries */
  db: DbExecutor;
  /** Optional network adapters for sync (Phase 2+) */
  network?: NetworkAdapterInterface[];
  /** Optional peer ID (generated if not provided) */
  peerId?: PeerId;
}

/**
 * Result of creating a new draft
 */
export interface CreateDraftResult {
  uuid: string;
  docId: DocumentId;
}

/**
 * SyncEngine manages Automerge documents for DashText.
 *
 * Responsibilities:
 * - Initialize and manage Automerge Repo
 * - Manage root document (draft list, metadata)
 * - Create and retrieve draft documents
 * - Provide subscriptions for reactive UI updates
 * - Maintain UUID → DocumentId mappings
 */
export class SyncEngine {
  private repo: Repo;
  private db: DbExecutor;
  private rootDocHandle: DocHandle<DashTextRoot> | null = null;
  private rootDocId: DocumentId | null = null;
  private draftHandles: Map<string, DocHandle<DashTextDraft>> = new Map();
  private initialized = false;

  constructor(config: SyncEngineConfig) {
    this.db = config.db;
    this.repo = new Repo({
      storage: config.storage,
      network: config.network ?? [],
      peerId: config.peerId,
    });
  }

  /**
   * Initialize the SyncEngine.
   *
   * Loads or creates the root document. Must be called before other operations.
   * This is idempotent - calling it multiple times is safe.
   */
  async initialize(): Promise<void> {
    if (this.initialized) return;

    // Check if we have an existing root doc ID in sync_state
    const syncStateRows = await this.db.select<{ root_doc_id: string | null }>(
      `SELECT root_doc_id FROM sync_state WHERE id = 1`
    );

    if (syncStateRows.length > 0 && syncStateRows[0].root_doc_id) {
      // Load existing root document
      this.rootDocId = syncStateRows[0].root_doc_id as DocumentId;
      this.rootDocHandle = await this.repo.find<DashTextRoot>(this.rootDocId);
      await this.rootDocHandle.whenReady();
    } else {
      // Create new root document
      this.rootDocHandle = this.repo.create<DashTextRoot>({
        schemaVersion: 1,
        drafts: {},
        pinnedUuid: null,
      });
      this.rootDocId = this.rootDocHandle.documentId;

      // Store root doc ID in sync_state
      const now = new Date().toISOString();
      await this.db.execute(
        `INSERT OR REPLACE INTO sync_state
         (id, sync_enabled, root_doc_id, created_at, updated_at)
         VALUES (1, 0, ?, ?, ?)`,
        [this.rootDocId, now, now]
      );
    }

    this.initialized = true;
  }

  /**
   * Ensure the engine is initialized before operations.
   */
  private assertInitialized(): void {
    if (!this.initialized) {
      throw new Error('SyncEngine not initialized. Call initialize() first.');
    }
  }

  /**
   * Get the root document handle.
   */
  getRootDocHandle(): DocHandle<DashTextRoot> {
    this.assertInitialized();
    return this.rootDocHandle!;
  }

  /**
   * Get the current state of the root document.
   */
  getRootDoc(): DashTextRoot {
    this.assertInitialized();
    return this.rootDocHandle!.doc();
  }

  /**
   * Create a new draft document.
   *
   * Creates both the Automerge document and the root document entry.
   * Returns the UUID and document ID for the new draft.
   */
  async createDraft(): Promise<CreateDraftResult> {
    this.assertInitialized();

    const uuid = crypto.randomUUID();
    const now = new Date().toISOString();

    // Create the draft document
    const draftHandle = this.repo.create<DashTextDraft>({
      schemaVersion: 1,
      uuid,
      content: '',
      createdAt: now,
      modifiedAt: now,
    });
    const docId = draftHandle.documentId;

    // Add entry to root document
    this.rootDocHandle!.change((doc) => {
      doc.drafts[uuid] = {
        docId,
        createdAt: now,
        modifiedAt: now,
        archived: false,
        pinned: false,
        deletedAt: null,
      };
    });

    // Store UUID → docId mapping
    await this.db.execute(
      `INSERT INTO automerge_doc_map (draft_uuid, doc_id, created_at)
       VALUES (?, ?, ?)`,
      [uuid, docId, now]
    );

    // Cache the handle
    this.draftHandles.set(uuid, draftHandle);

    return { uuid, docId };
  }

  /**
   * Get the document handle for a draft by UUID.
   *
   * Loads from storage if not already cached.
   */
  async getDraftDocHandle(uuid: string): Promise<DocHandle<DashTextDraft> | null> {
    this.assertInitialized();

    // Check cache first
    if (this.draftHandles.has(uuid)) {
      return this.draftHandles.get(uuid)!;
    }

    // Look up the document ID
    const rows = await this.db.select<DocMapRow>(
      `SELECT doc_id FROM automerge_doc_map WHERE draft_uuid = ?`,
      [uuid]
    );

    if (rows.length === 0) {
      return null;
    }

    const docId = rows[0].doc_id as DocumentId;
    const handle = await this.repo.find<DashTextDraft>(docId);
    await handle.whenReady();

    // Cache the handle
    this.draftHandles.set(uuid, handle);

    return handle;
  }

  /**
   * Get the current state of a draft document.
   */
  async getDraftDoc(uuid: string): Promise<DashTextDraft | null> {
    const handle = await this.getDraftDocHandle(uuid);
    return handle?.doc() ?? null;
  }

  /**
   * Update the content of a draft using splice operations.
   *
   * This is the preferred method for editor integration as it preserves
   * CRDT properties for concurrent edits.
   *
   * @param uuid - Draft UUID
   * @param index - Position in the content string
   * @param deleteCount - Number of characters to delete
   * @param insertText - Text to insert (optional)
   */
  async spliceDraftContent(
    uuid: string,
    index: number,
    deleteCount: number,
    insertText?: string
  ): Promise<void> {
    const handle = await this.getDraftDocHandle(uuid);
    if (!handle) {
      throw new Error(`Draft not found: ${uuid}`);
    }

    const now = new Date().toISOString();

    handle.change((doc) => {
      // Use Automerge's splice for text editing
      Automerge.splice(doc, ['content'], index, deleteCount, insertText);
      doc.modifiedAt = now;
    });

    // Update root document's modifiedAt for this draft
    this.rootDocHandle!.change((doc) => {
      if (doc.drafts[uuid]) {
        doc.drafts[uuid].modifiedAt = now;
      }
    });
  }

  /**
   * Replace the entire content of a draft.
   *
   * Use spliceDraftContent for incremental edits when possible.
   * This method is useful for imports or bulk operations.
   */
  async setDraftContent(uuid: string, content: string): Promise<void> {
    const handle = await this.getDraftDocHandle(uuid);
    if (!handle) {
      throw new Error(`Draft not found: ${uuid}`);
    }

    const now = new Date().toISOString();

    handle.change((doc) => {
      const currentLength = doc.content.length;
      // Delete all existing content and insert new content
      Automerge.splice(doc, ['content'], 0, currentLength, content);
      doc.modifiedAt = now;
    });

    // Update root document's modifiedAt
    this.rootDocHandle!.change((doc) => {
      if (doc.drafts[uuid]) {
        doc.drafts[uuid].modifiedAt = now;
      }
    });
  }

  /**
   * Update draft metadata (archived, pinned, deleted).
   */
  async updateDraftMetadata(
    uuid: string,
    updates: Partial<Pick<DraftMetadata, 'archived' | 'pinned' | 'deletedAt'>>
  ): Promise<void> {
    this.assertInitialized();

    const now = new Date().toISOString();

    this.rootDocHandle!.change((doc) => {
      const meta = doc.drafts[uuid];
      if (!meta) return;

      if (updates.archived !== undefined) {
        meta.archived = updates.archived;
      }

      if (updates.pinned !== undefined) {
        // Enforce single-pin constraint
        if (updates.pinned) {
          // Unpin any currently pinned draft
          if (doc.pinnedUuid && doc.pinnedUuid !== uuid) {
            const prevPinned = doc.drafts[doc.pinnedUuid];
            if (prevPinned) {
              prevPinned.pinned = false;
            }
          }
          doc.pinnedUuid = uuid;
        } else if (doc.pinnedUuid === uuid) {
          doc.pinnedUuid = null;
        }
        meta.pinned = updates.pinned;
      }

      if (updates.deletedAt !== undefined) {
        meta.deletedAt = updates.deletedAt;
      }

      meta.modifiedAt = now;
    });
  }

  /**
   * Subscribe to changes on the root document.
   *
   * Returns an unsubscribe function.
   */
  onRootChange(callback: (doc: DashTextRoot) => void): () => void {
    this.assertInitialized();

    const handler = ({ doc }: { doc: DashTextRoot }) => {
      callback(doc);
    };

    this.rootDocHandle!.on('change', handler);

    return () => {
      this.rootDocHandle!.off('change', handler);
    };
  }

  /**
   * Subscribe to changes on a specific draft document.
   *
   * Returns an unsubscribe function.
   */
  async onDraftChange(
    uuid: string,
    callback: (doc: DashTextDraft) => void
  ): Promise<() => void> {
    const handle = await this.getDraftDocHandle(uuid);
    if (!handle) {
      throw new Error(`Draft not found: ${uuid}`);
    }

    const handler = ({ doc }: { doc: DashTextDraft }) => {
      callback(doc);
    };

    handle.on('change', handler);

    return () => {
      handle.off('change', handler);
    };
  }

  /**
   * Get all draft UUIDs from the root document.
   *
   * Optionally filter by status (active, archived, deleted).
   */
  getDraftUuids(filter?: {
    archived?: boolean;
    deleted?: boolean;
  }): string[] {
    this.assertInitialized();

    const root = this.getRootDoc();
    const uuids: string[] = [];

    for (const [uuid, meta] of Object.entries(root.drafts)) {
      const isDeleted = meta.deletedAt != null;
      const isArchived = meta.archived;

      if (filter?.deleted !== undefined && isDeleted !== filter.deleted) {
        continue;
      }
      if (filter?.archived !== undefined && isArchived !== filter.archived) {
        continue;
      }

      uuids.push(uuid);
    }

    return uuids;
  }

  /**
   * Get draft metadata from the root document.
   */
  getDraftMetadata(uuid: string): DraftMetadata | null {
    this.assertInitialized();
    return this.getRootDoc().drafts[uuid] ?? null;
  }

  /**
   * Check if the engine is initialized.
   */
  isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Get the underlying Repo instance (for advanced use cases).
   */
  getRepo(): Repo {
    return this.repo;
  }
}
