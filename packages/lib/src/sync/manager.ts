import { SyncEngine } from './SyncEngine';
import { SqliteRepoStorageAdapter } from './storage/SqliteRepoStorageAdapter';
import type { DbExecutor } from './storage/db-executor';

let instance: SyncEngine | null = null;

/**
 * Get or create the singleton SyncEngine instance.
 * 
 * This ensures a single SyncEngine instance across the application,
 * which is required for proper Automerge document management.
 */
export function getSyncEngine(db: DbExecutor): SyncEngine {
  if (instance) return instance;

  const storage = new SqliteRepoStorageAdapter(db);
  instance = new SyncEngine({ storage, db });
  
  return instance;
}

/**
 * Initialize the SyncEngine if not already initialized.
 * 
 * Must be called before using SyncEngine for any operations.
 * This is idempotent - calling it multiple times is safe.
 */
export async function initializeSyncEngine(db: DbExecutor): Promise<SyncEngine> {
  const syncEngine = getSyncEngine(db);
  
  if (!syncEngine.isInitialized()) {
    await syncEngine.initialize();
  }
  
  return syncEngine;
}
