import Database from '@tauri-apps/plugin-sql';
import { drizzle } from 'drizzle-orm/sqlite-proxy';
import { drafts, type Draft, type NewDraft, settings, type Settings, type NewSettings } from '@dashtext/lib/db';
import type { DbExecutor } from '@dashtext/lib/sync';
import { getSyncEngine, initializeSyncEngine, migrateExistingDrafts } from '@dashtext/lib/sync';

export { drafts, type Draft, type NewDraft, settings, type Settings, type NewSettings };

const DB_URL = 'sqlite:dashtext.db';

let db: ReturnType<typeof drizzle> | null = null;
let sqliteInstance: Database | null = null;

/**
 * Checks if the given SQL query returns rows (SELECT or RETURNING clause).
 */
function isSelectQuery(sql: string): boolean {
  return /^\s*SELECT\b/i.test(sql) || /\bRETURNING\b/i.test(sql);
}

/**
 * Get the Drizzle database instance.
 * Lazily initializes the connection on first call.
 */
export async function getDb() {
  if (db) return db;

  const sqlite = await Database.load(DB_URL);
  sqliteInstance = sqlite;

  db = drizzle(
    async (sql, params, method) => {
      try {
        if (isSelectQuery(sql)) {
          const result = await sqlite.select<Record<string, unknown>[]>(sql, params as unknown[]);
          const rows = result.map((row) => Object.values(row));
          return { rows: method === 'all' ? rows : rows[0] } as { rows: unknown[] };
        }
        await sqlite.execute(sql, params as unknown[]);
        return { rows: [] };
      } catch (e) {
        console.error('Database error:', e);
        throw e;
      }
    },
    { schema: { drafts, settings } }
  );

  const dbExecutor = new TauriDbExecutor(sqlite);
  const syncEngine = await initializeSyncEngine(dbExecutor);
  await migrateExistingDrafts(syncEngine, dbExecutor);

  return db;
}

/**
 * Tauri-specific DbExecutor implementation.
 */
class TauriDbExecutor implements DbExecutor {
  constructor(private sqlite: Database) {}

  async select<T>(sql: string, params?: unknown[]): Promise<T[]> {
    const result = await this.sqlite.select(sql, params as unknown[]);
    return result as T[];
  }

  async execute(sql: string, params?: unknown[]): Promise<void> {
    await this.sqlite.execute(sql, params as unknown[]);
  }
}

/**
 * Get the SyncEngine instance.
 * Must call getDb() first to initialize the database.
 */
export function getSyncEngineInstance() {
  if (!sqliteInstance) {
    throw new Error('Database not initialized. Call getDb() first.');
  }
  return getSyncEngine(new TauriDbExecutor(sqliteInstance));
}
