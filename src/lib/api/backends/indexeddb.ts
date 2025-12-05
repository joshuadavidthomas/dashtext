import { openDB, type DBSchema, type IDBPDatabase } from 'idb';
import type { DraftAPI, DraftData } from '../types';

interface DashtextDB extends DBSchema {
	drafts: {
		key: number;
		value: DraftData;
		indexes: {
			'by-modified': string;
		};
	};
}

const DB_NAME = 'dashtext';
const DB_VERSION = 1;

let dbPromise: Promise<IDBPDatabase<DashtextDB>> | null = null;

function getDB(): Promise<IDBPDatabase<DashtextDB>> {
	if (!dbPromise) {
		dbPromise = openDB<DashtextDB>(DB_NAME, DB_VERSION, {
			upgrade(db) {
				const store = db.createObjectStore('drafts', {
					keyPath: 'id',
					autoIncrement: true
				});
				store.createIndex('by-modified', 'modified_at');
			}
		});
	}
	return dbPromise;
}

/**
 * IndexedDB backend for web browser persistence.
 * Data is stored locally in the browser - no sync between devices.
 */
class IndexedDBBackend implements DraftAPI {
	async list(): Promise<DraftData[]> {
		const db = await getDB();
		const drafts = await db.getAll('drafts');
		// Sort by modified_at descending (most recent first)
		return drafts.sort(
			(a, b) => new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime()
		);
	}

	async create(): Promise<DraftData> {
		const db = await getDB();
		const now = new Date().toISOString();
		const draft: Omit<DraftData, 'id'> = {
			content: '',
			created_at: now,
			modified_at: now
		};

		const id = await db.add('drafts', draft as DraftData);
		return { ...draft, id: id as number };
	}

	async get(id: number): Promise<DraftData | null> {
		const db = await getDB();
		const draft = await db.get('drafts', id);
		return draft ?? null;
	}

	async save(id: number, content: string): Promise<DraftData> {
		const db = await getDB();
		const draft = await db.get('drafts', id);

		if (!draft) {
			throw new Error(`Draft ${id} not found`);
		}

		const updated: DraftData = {
			...draft,
			content,
			modified_at: new Date().toISOString()
		};

		await db.put('drafts', updated);
		return updated;
	}

	async delete(id: number): Promise<void> {
		const db = await getDB();
		await db.delete('drafts', id);
	}
}

export const indexedDBBackend: DraftAPI = new IndexedDBBackend();
