/**
 * Platform-agnostic database executor interface.
 *
 * Abstracts SQL operations to work on both:
 * - Desktop: Tauri plugin-sql (native SQLite)
 * - Web: sql.js (WASM SQLite in browser)
 *
 * Implementations should handle BLOB data (Uint8Array) correctly.
 */
export interface DbExecutor {
  /**
   * Execute a SELECT query and return rows.
   * @param sql SQL query string with ? placeholders
   * @param params Query parameters
   * @returns Array of row objects
   */
  select<T = Record<string, unknown>>(sql: string, params?: unknown[]): Promise<T[]>;

  /**
   * Execute a non-SELECT query (INSERT, UPDATE, DELETE).
   * @param sql SQL statement with ? placeholders
   * @param params Query parameters
   */
  execute(sql: string, params?: unknown[]): Promise<void>;
}

/**
 * Row type for automerge_chunk table
 */
export interface ChunkRow {
  doc_id: string;
  chunk_type: string;
  chunk_id: string;
  bytes: Uint8Array;
  created_at: string;
}

/**
 * Row type for automerge_doc_map table
 */
export interface DocMapRow {
  draft_uuid: string;
  doc_id: string;
  created_at: string;
}

/**
 * Row type for sync_state table
 */
export interface SyncStateRow {
  id: number;
  sync_enabled: number; // SQLite boolean as integer
  space_id: string | null;
  device_id: string | null;
  auth_token: string | null;
  server_url: string | null;
  root_doc_id: string | null;
  last_connected_at: string | null;
  created_at: string;
  updated_at: string;
}
