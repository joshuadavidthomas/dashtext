import { openDB, type DBSchema, type IDBPDatabase } from 'idb';
import type { Database, SqlJsStatic } from './sqljs';

interface SqliteDB extends DBSchema {
	databases: {
		key: string;
		value: {
			name: string;
			data: Uint8Array;
			updatedAt: number;
		};
	};
}

const IDB_NAME = 'dashtext-sqlite';
const IDB_VERSION = 1;
const DB_NAME = 'dashtext';

let idb: IDBPDatabase<SqliteDB> | null = null;

/**
 * Get the IndexedDB connection, creating it if necessary
 */
async function getIDB(): Promise<IDBPDatabase<SqliteDB>> {
	if (idb) return idb;

	idb = await openDB<SqliteDB>(IDB_NAME, IDB_VERSION, {
		upgrade(db) {
			if (!db.objectStoreNames.contains('databases')) {
				db.createObjectStore('databases', { keyPath: 'name' });
			}
		},
	});

	return idb;
}

/**
 * Save SQLite database to IndexedDB
 */
export async function saveDatabase(sqlite: Database, dbName: string = DB_NAME): Promise<void> {
	const idb = await getIDB();
	const data = sqlite.export();

	await idb.put('databases', {
		name: dbName,
		data,
		updatedAt: Date.now(),
	});
}

/**
 * Load SQLite database from IndexedDB
 */
export async function loadDatabase(SQL: SqlJsStatic, dbName: string = DB_NAME): Promise<Database> {
	const idb = await getIDB();
	const stored = await idb.get('databases', dbName);

	if (stored?.data) {
		return new SQL.Database(stored.data);
	}

	return new SQL.Database();
}

/**
 * Create debounced auto-save function
 */
export function createAutoSave(sqlite: Database, dbName: string = DB_NAME, delay = 1000) {
	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	return function save() {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}

		timeoutId = setTimeout(() => {
			saveDatabase(sqlite, dbName);
			timeoutId = null;
		}, delay);
	};
}
