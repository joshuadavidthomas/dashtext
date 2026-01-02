/**
 * Automerge document schemas for DashText sync
 *
 * Note: In Automerge 3.x, text content is stored as a string field and
 * edited using splice() at the document level rather than Text class methods.
 * See: https://automerge.org/docs/under-the-hood/rich_text_schema/
 */

/**
 * Root document containing all draft metadata and ordering.
 * One root document per space.
 */
export interface DashTextRoot {
  schemaVersion: 1;
  drafts: {
    [draftUuid: string]: DraftMetadata;
  };
  /** UUID of the currently pinned draft (single pin constraint) */
  pinnedUuid?: string | null;
}

/**
 * Metadata for a draft stored in the root document.
 * This is the source of truth for list ordering and state.
 */
export interface DraftMetadata {
  /** Automerge document ID for this draft's content */
  docId: string;
  createdAt: string;
  modifiedAt: string;
  archived: boolean;
  pinned: boolean;
  deletedAt?: string | null;
}

/**
 * Individual draft document containing the actual content.
 * One document per draft.
 *
 * Content is stored as a string but should be edited using Automerge's
 * splice() function for proper CRDT behavior:
 *
 * ```ts
 * import * as Automerge from '@automerge/automerge';
 * Automerge.splice(doc, ['content'], index, deleteCount, insertText);
 * ```
 */
export interface DashTextDraft {
  schemaVersion: 1;
  uuid: string;
  /** Text content - use splice() for edits, not direct assignment */
  content: string;
  createdAt: string;
  /** Client-updated timestamp; root doc is canonical for list ordering */
  modifiedAt: string;
}

/**
 * Sync configuration stored locally
 */
export interface SyncConfig {
  syncEnabled: boolean;
  spaceId?: string;
  deviceId?: string;
  authToken?: string;
  serverUrl?: string;
  rootDocId?: string;
  lastConnectedAt?: string;
}

/**
 * Sync connection status
 */
export type SyncStatus = 'disconnected' | 'connecting' | 'connected' | 'syncing' | 'error';
