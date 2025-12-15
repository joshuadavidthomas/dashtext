import { openDB, type DBSchema, type IDBPDatabase } from 'idb';
import type { DraftAPI, DraftData } from '@dashtext/lib';
import { generateUUID } from '@dashtext/lib';

interface DashtextDB extends DBSchema {
  drafts: {
    key: number;
    value: {
      id: number;
      uuid: string;
      content: string;
      created_at: string;
      modified_at: string;
      deleted_at?: string;
      archived?: boolean;
      pinned?: boolean;
    };
    indexes: {
      'by-uuid': string;
      'by-modified': string;
    };
  };
}

const DB_NAME = 'dashtext';
const DB_VERSION = 2; // Increment for schema change

let dbPromise: Promise<IDBPDatabase<DashtextDB>> | null = null;

function getDB(): Promise<IDBPDatabase<DashtextDB>> {
  if (!dbPromise) {
    dbPromise = openDB<DashtextDB>(DB_NAME, DB_VERSION, {
      upgrade(db, oldVersion) {
        if (oldVersion < 2) {
          const store = db.createObjectStore('drafts', {
            keyPath: 'id',
            autoIncrement: true
          });
          store.createIndex('by-uuid', 'uuid', { unique: true });
          store.createIndex('by-modified', 'modified_at');
        }
      }
    });
  }
  return dbPromise;
}

class IndexedDBBackend implements DraftAPI {
  async list(): Promise<DraftData[]> {
    const db = await getDB();
    const drafts = await db.getAll('drafts');
    return drafts
      .filter(draft => !draft.deleted_at && !draft.archived)
      .sort((a, b) => {
        // Pinned drafts first
        if (a.pinned && !b.pinned) return -1;
        if (!a.pinned && b.pinned) return 1;
        // Then by modified date
        return new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime();
      })
      .map(({ id, ...draft }) => draft); // Remove internal id
  }

  async create(): Promise<DraftData> {
    const db = await getDB();
    const now = new Date().toISOString();
    const draft = {
      uuid: generateUUID(),
      content: '',
      created_at: now,
      modified_at: now,
      archived: false,
      pinned: false,
      id: 0 // Temporary placeholder, will be replaced by auto-increment
    };

    const id = await db.add('drafts', draft);
    const { id: internalId, ...publicDraft } = { ...draft, id } as any;
    return publicDraft;
  }

  async get(uuid: string): Promise<DraftData | null> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readonly');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    await tx.done;
    
    return draft ? { uuid: draft.uuid, content: draft.content, created_at: draft.created_at, modified_at: draft.modified_at, deleted_at: draft.deleted_at, archived: draft.archived, pinned: draft.pinned } : null;
  }

  async save(uuid: string, content: string): Promise<DraftData> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);

    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    const updated = {
      ...draft,
      content,
      modified_at: new Date().toISOString()
    };

    await tx.objectStore('drafts').put(updated);
    await tx.done;
    
    const { id, ...publicDraft } = updated;
    return publicDraft;
  }

  async archive(uuid: string): Promise<DraftData> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    const updated = { ...draft, archived: true };
    await tx.objectStore('drafts').put(updated);
    await tx.done;
    
    const { id: _, ...publicDraft } = updated;
    return publicDraft;
  }

  async unarchive(uuid: string): Promise<DraftData> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    const updated = { ...draft, archived: false };
    await tx.objectStore('drafts').put(updated);
    await tx.done;
    
    const { id: _, ...publicDraft } = updated;
    return publicDraft;
  }

  async pin(uuid: string): Promise<DraftData> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const store = tx.objectStore('drafts');
    
    // Unpin all other drafts first (single pin constraint)
    const drafts = await store.getAll();
    for (const draft of drafts) {
      if (draft.pinned) {
        await store.put({ ...draft, pinned: false });
      }
    }
    
    // Pin the requested draft
    const index = store.index('by-uuid');
    const draft = await index.get(uuid);
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }
    
    const updated = { ...draft, pinned: true };
    await store.put(updated);
    await tx.done;
    
    const { id: _, ...publicDraft } = updated;
    return publicDraft;
  }

  async unpin(uuid: string): Promise<DraftData> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    const updated = { ...draft, pinned: false };
    await tx.objectStore('drafts').put(updated);
    await tx.done;
    
    const { id: _, ...publicDraft } = updated;
    return publicDraft;
  }

  async restore(uuid: string): Promise<DraftData> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    const updated = { ...draft, deleted_at: undefined };
    await tx.objectStore('drafts').put(updated);
    await tx.done;
    
    const { id: _, ...publicDraft } = updated;
    return publicDraft;
  }

  async delete(uuid: string): Promise<void> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    const updated = { ...draft, deleted_at: new Date().toISOString() };
    await tx.objectStore('drafts').put(updated);
    await tx.done;
  }

  async hardDelete(uuid: string): Promise<void> {
    const db = await getDB();
    const tx = db.transaction('drafts', 'readwrite');
    const index = tx.objectStore('drafts').index('by-uuid');
    const draft = await index.get(uuid);
    
    if (!draft) {
      await tx.abort();
      throw new Error(`Draft ${uuid} not found`);
    }

    await tx.objectStore('drafts').delete(draft.id);
    await tx.done;
  }
}

export default new IndexedDBBackend();
