import Database from '@tauri-apps/plugin-sql';
import { drizzle } from 'drizzle-orm/sqlite-proxy';
import * as schema from './schema';

export * from './schema';

let db: ReturnType<typeof drizzle<typeof schema>> | null = null;
let sqliteDb: Database | null = null;

/**
 * Checks if the given SQL query is a SELECT query (or INSERT/UPDATE/DELETE with RETURNING).
 */
function isSelectQuery(sql: string): boolean {
  const selectRegex = /^\s*SELECT\b/i;
  const returningRegex = /\bRETURNING\b/i;
  return selectRegex.test(sql) || returningRegex.test(sql);
}

export async function getDb() {
  if (db) return db;

  // Load the database - this connects to the preloaded instance from tauri.conf.json
  sqliteDb = await Database.load('sqlite:dashtext.db');

  db = drizzle<typeof schema>(
    async (sql, params, method) => {
      let rows: unknown[] = [];

      // If the query is a SELECT (or has RETURNING), use the select method
      if (isSelectQuery(sql)) {
        const result = await sqliteDb!.select<Record<string, unknown>[]>(sql, params as unknown[]).catch((e) => {
          console.error('SQL Select Error:', e, { sql, params });
          return [];
        });

        // Transform rows from objects to arrays of values (required by sqlite-proxy)
        rows = result.map((row) => Object.values(row));
      } else {
        // For INSERT/UPDATE/DELETE without RETURNING, use execute
        await sqliteDb!.execute(sql, params as unknown[]).catch((e) => {
          console.error('SQL Execute Error:', e, { sql, params });
        });
        return { rows: [] };
      }

      // Handle 'all' vs 'get' method - 'all' returns all rows, 'get' returns first row
      const results = method === 'all' ? rows : rows[0];

      return { rows: results as unknown[] };
    },
    { schema, logger: true }
  );

  return db;
}
