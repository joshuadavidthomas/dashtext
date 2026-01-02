/**
 * SQLite-backed storage adapter for Automerge Repo.
 *
 * Works on both desktop (Tauri plugin-sql) and web (sql.js) via the
 * DbExecutor abstraction.
 *
 * Storage key format: [docId, chunkType, chunkId]
 * - docId: Automerge document ID
 * - chunkType: "snapshot" | "incremental"
 * - chunkId: Hash or identifier for the chunk
 */

import type { StorageAdapterInterface } from '@automerge/automerge-repo';
import type { Chunk, StorageKey } from '@automerge/automerge-repo';
import type { DbExecutor, ChunkRow } from './db-executor';

export class SqliteRepoStorageAdapter implements StorageAdapterInterface {
  private db: DbExecutor;

  constructor(db: DbExecutor) {
    this.db = db;
  }

  /**
   * Load a single chunk by its key.
   */
  async load(key: StorageKey): Promise<Uint8Array | undefined> {
    const [docId, chunkType, chunkId] = key;

    if (!docId || !chunkType || !chunkId) {
      return undefined;
    }

    const rows = await this.db.select<ChunkRow>(
      `SELECT bytes FROM automerge_chunk
       WHERE doc_id = ? AND chunk_type = ? AND chunk_id = ?`,
      [docId, chunkType, chunkId]
    );

    if (rows.length === 0) {
      return undefined;
    }

    // Handle potential format differences between platforms
    return this.toUint8Array(rows[0].bytes);
  }

  /**
   * Save a chunk to storage.
   */
  async save(key: StorageKey, data: Uint8Array): Promise<void> {
    const [docId, chunkType, chunkId] = key;

    if (!docId || !chunkType || !chunkId) {
      throw new Error(`Invalid storage key: ${JSON.stringify(key)}`);
    }

    const now = new Date().toISOString();

    // Use INSERT OR REPLACE to handle both insert and update
    await this.db.execute(
      `INSERT OR REPLACE INTO automerge_chunk
       (doc_id, chunk_type, chunk_id, bytes, created_at)
       VALUES (?, ?, ?, ?, ?)`,
      [docId, chunkType, chunkId, data, now]
    );
  }

  /**
   * Remove a single chunk by its key.
   */
  async remove(key: StorageKey): Promise<void> {
    const [docId, chunkType, chunkId] = key;

    if (!docId || !chunkType || !chunkId) {
      return;
    }

    await this.db.execute(
      `DELETE FROM automerge_chunk
       WHERE doc_id = ? AND chunk_type = ? AND chunk_id = ?`,
      [docId, chunkType, chunkId]
    );
  }

  /**
   * Load all chunks matching a key prefix.
   *
   * Key prefix examples:
   * - [docId] - all chunks for a document
   * - [docId, "snapshot"] - all snapshots for a document
   * - [docId, "incremental"] - all incremental changes for a document
   */
  async loadRange(keyPrefix: StorageKey): Promise<Chunk[]> {
    const [docId, chunkType] = keyPrefix;

    if (!docId) {
      // Empty prefix - return all chunks (expensive, avoid in practice)
      const rows = await this.db.select<ChunkRow>(
        `SELECT doc_id, chunk_type, chunk_id, bytes FROM automerge_chunk`
      );
      return rows.map((row) => this.rowToChunk(row));
    }

    if (!chunkType) {
      // Just docId - return all chunks for this document
      const rows = await this.db.select<ChunkRow>(
        `SELECT doc_id, chunk_type, chunk_id, bytes FROM automerge_chunk
         WHERE doc_id = ?
         ORDER BY chunk_type, chunk_id`,
        [docId]
      );
      return rows.map((row) => this.rowToChunk(row));
    }

    // Both docId and chunkType
    const rows = await this.db.select<ChunkRow>(
      `SELECT doc_id, chunk_type, chunk_id, bytes FROM automerge_chunk
       WHERE doc_id = ? AND chunk_type = ?
       ORDER BY chunk_id`,
      [docId, chunkType]
    );
    return rows.map((row) => this.rowToChunk(row));
  }

  /**
   * Remove all chunks matching a key prefix.
   */
  async removeRange(keyPrefix: StorageKey): Promise<void> {
    const [docId, chunkType] = keyPrefix;

    if (!docId) {
      // Empty prefix - remove all (dangerous, but valid)
      await this.db.execute(`DELETE FROM automerge_chunk`);
      return;
    }

    if (!chunkType) {
      // Just docId - remove all chunks for this document
      await this.db.execute(`DELETE FROM automerge_chunk WHERE doc_id = ?`, [docId]);
      return;
    }

    // Both docId and chunkType
    await this.db.execute(
      `DELETE FROM automerge_chunk WHERE doc_id = ? AND chunk_type = ?`,
      [docId, chunkType]
    );
  }

  /**
   * Convert a database row to a Chunk.
   */
  private rowToChunk(row: ChunkRow): Chunk {
    return {
      key: [row.doc_id, row.chunk_type, row.chunk_id],
      data: this.toUint8Array(row.bytes),
    };
  }

  /**
   * Ensure bytes are in Uint8Array format.
   *
   * Different platforms may return BLOB data in different formats:
   * - Tauri plugin-sql: may return ArrayBuffer or Uint8Array
   * - sql.js: returns Uint8Array directly
   * - Some platforms may return base64 strings
   */
  private toUint8Array(data: unknown): Uint8Array {
    if (data instanceof Uint8Array) {
      return data;
    }

    if (data instanceof ArrayBuffer) {
      return new Uint8Array(data);
    }

    if (Array.isArray(data)) {
      // Some drivers return arrays of numbers
      return new Uint8Array(data);
    }

    if (typeof data === 'string') {
      // Assume base64 encoding
      const binaryString = atob(data);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      return bytes;
    }

    // Handle Buffer (Node.js)
    if (data && typeof data === 'object' && 'buffer' in data) {
      const buffer = data as { buffer: ArrayBuffer; byteOffset: number; byteLength: number };
      return new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    }

    throw new Error(`Unexpected bytes format: ${typeof data}`);
  }
}
