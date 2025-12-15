import initSqlJs, { type SqlJsStatic, type Database } from 'sql.js';

let SQL: SqlJsStatic | null = null;

/**
 * Get the sql.js library instance.
 * Lazily initializes on first call and caches for reuse.
 */
export async function getSqlJs(): Promise<SqlJsStatic> {
	if (SQL) return SQL;

	SQL = await initSqlJs({
		locateFile: (file) => `/sql.js/${file}`,
	});

	return SQL;
}

export type { Database, SqlJsStatic };
