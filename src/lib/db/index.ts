import Database from '@tauri-apps/plugin-sql';
import { drizzle } from 'drizzle-orm/sqlite-proxy';
import * as schema from './schema';

export * from './schema';

const DB_URL = 'sqlite:dashtext.db';

let db: ReturnType<typeof drizzle<typeof schema>> | null = null;

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

  db = drizzle<typeof schema>(
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
    { schema }
  );

  return db;
}
