import { drizzle } from 'drizzle-orm/sql-js';
import { drafts } from '@dashtext/lib/db';
import { migrations } from '@dashtext/lib/db/migrations';
import { getSqlJs, type Database } from './sqljs';
import { loadDatabase, saveDatabase, createAutoSave } from './persist';
import type { DbExecutor } from '@dashtext/lib/sync';
import { getSyncEngine, initializeSyncEngine, migrateExistingDrafts } from '@dashtext/lib/sync';

type DrizzleDB = ReturnType<typeof drizzle>;

let db: DrizzleDB | null = null;
let sqlite: Database | null = null;
let autoSave: (() => void) | null = null;

/**
 * Run migrations on the database
 */
function runMigrations(sqliteDb: Database): void {
	// Check if migrations table exists
	const hasMigrationsTable = sqliteDb.exec(`
		SELECT name FROM sqlite_master 
		WHERE type='table' AND name='__drizzle_migrations'
	`);

	if (hasMigrationsTable.length === 0) {
		// Create migrations table
		sqliteDb.exec(`
			CREATE TABLE __drizzle_migrations (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				hash TEXT NOT NULL UNIQUE,
				created_at INTEGER
			)
		`);
	}

	// Get applied migrations
	const appliedMigrations = sqliteDb.exec(
		'SELECT hash FROM __drizzle_migrations ORDER BY id'
	);
	const appliedHashes = new Set(
		appliedMigrations.length > 0 
			? appliedMigrations[0].values.map((row) => row[0] as string)
			: []
	);

	// Apply pending migrations
	for (const migration of migrations) {
		if (!appliedHashes.has(migration.hash)) {
			console.log(`[Migrations] Applying migration ${migration.hash}`);
			
			// Run each SQL statement
			for (const statement of migration.sql) {
				if (statement.trim()) {
					sqliteDb.exec(statement);
				}
			}

			// Record migration
			sqliteDb.exec(
				`INSERT INTO __drizzle_migrations (hash, created_at) VALUES (?, ?)`,
				[migration.hash, Date.now()]
			);

			console.log(`[Migrations] Applied ${migration.hash}`);
		}
	}
}

/**
 * Get the Drizzle database instance.
 * Lazily initializes on first call and caches for reuse.
 */
export async function getDb(): Promise<DrizzleDB> {
	if (db) return db;

	// Initialize sql.js and load database from IndexedDB
	const SQL = await getSqlJs();
	sqlite = await loadDatabase(SQL);

	// Run migrations
	runMigrations(sqlite);

	// Create Drizzle instance
	db = drizzle(sqlite, { schema: { drafts } });

	// Set up auto-save to IndexedDB
	autoSave = createAutoSave(sqlite);

	// Save on visibility change and beforeunload
	if (typeof document !== 'undefined') {
		document.addEventListener('visibilitychange', () => {
			if (document.visibilityState === 'hidden' && sqlite) {
				saveDatabase(sqlite);
			}
		});

		window.addEventListener('beforeunload', () => {
			if (sqlite) {
				saveDatabase(sqlite);
			}
		});
	}

	// Initialize SyncEngine after DB is ready
	const dbExecutor = new SqlJsDbExecutor(sqlite);
	const syncEngine = await initializeSyncEngine(dbExecutor);
	await migrateExistingDrafts(syncEngine, dbExecutor);

	return db;
}

/**
 * sql.js-based DbExecutor implementation.
 */
class SqlJsDbExecutor implements DbExecutor {
	constructor(private sqlite: Database) {}

	async select<T>(sql: string, params?: unknown[]): Promise<T[]> {
		const results = this.sqlite.exec(sql, params as any);
		if (!results || results.length === 0) return [];

		const queryResult = results[0];
		const columns = queryResult.columns;
		const values = queryResult.values;

		return values.map((row) => {
			const obj: Record<string, unknown> = {};
			columns.forEach((col: string, i: number) => {
				obj[col] = row[i];
			});
			return obj as T;
		});
	}

	async execute(sql: string, params?: unknown[]): Promise<void> {
		this.sqlite.exec(sql, params as any);
	}
}

/**
 * Get the SyncEngine instance.
 * Must call getDb() first to initialize the database.
 */
export function getSyncEngineInstance() {
	if (!sqlite) {
		throw new Error('Database not initialized. Call getDb() first.');
	}
	return getSyncEngine(new SqlJsDbExecutor(sqlite));
}

/**
 * Manually trigger a save to IndexedDB
 */
export async function saveDb(): Promise<void> {
	if (sqlite) {
		await saveDatabase(sqlite);
	}
}

/**
 * Trigger the debounced auto-save
 */
export function triggerAutoSave(): void {
	if (autoSave) {
		autoSave();
	}
}

export { drafts };
export type { DrizzleDB };
