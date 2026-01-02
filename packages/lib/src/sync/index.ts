// Storage
export { SqliteRepoStorageAdapter } from './storage/SqliteRepoStorageAdapter';
export type { DbExecutor, ChunkRow, DocMapRow, SyncStateRow } from './storage/db-executor';

// Manager
export { getSyncEngine, initializeSyncEngine } from './manager';

// Migration
export { needsMigration, migrateExistingDrafts } from './migration';

// Types
export type {
  DashTextRoot,
  DashTextDraft,
  DraftMetadata,
  SyncConfig,
  SyncStatus,
} from './types';

// Core
export { SyncEngine } from './SyncEngine';
export type { SyncEngineConfig, CreateDraftResult } from './SyncEngine';
