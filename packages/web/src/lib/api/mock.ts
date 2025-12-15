import type { DraftAPI, DraftData } from '@dashtext/lib';
import { generateUUID } from '@dashtext/lib';

// Mock backend using UUID as primary key with an internal map
const drafts: Map<string, DraftData> = new Map();

function getNow(): string {
  return new Date().toISOString();
}

export const mockBackend: DraftAPI = {
  async list(): Promise<DraftData[]> {
    return Array.from(drafts.values()).sort(
      (a, b) => new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime()
    );
  },

  async create(): Promise<DraftData> {
    const now = getNow();
    const draft: DraftData = {
      uuid: generateUUID(),
      content: '',
      created_at: now,
      modified_at: now
    };
    drafts.set(draft.uuid, draft);
    return draft;
  },

  async get(uuid: string): Promise<DraftData | null> {
    return drafts.get(uuid) ?? null;
  },

  async save(uuid: string, content: string): Promise<DraftData> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);

    const updated = { ...draft, content, modified_at: getNow() };
    drafts.set(uuid, updated);
    return updated;
  },

  async archive(uuid: string): Promise<DraftData> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);
    
    const updated = { ...draft, archived: true };
    drafts.set(uuid, updated);
    return updated;
  },

  async unarchive(uuid: string): Promise<DraftData> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);
    
    const updated = { ...draft, archived: false };
    drafts.set(uuid, updated);
    return updated;
  },

  async pin(uuid: string): Promise<DraftData> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);
    
    // Unpin all others first
    for (const [key, d] of drafts) {
      if (d.pinned) {
        drafts.set(key, { ...d, pinned: false });
      }
    }
    
    const updated = { ...draft, pinned: true };
    drafts.set(uuid, updated);
    return updated;
  },

  async unpin(uuid: string): Promise<DraftData> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);
    
    const updated = { ...draft, pinned: false };
    drafts.set(uuid, updated);
    return updated;
  },

  async restore(uuid: string): Promise<DraftData> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);
    
    const updated = { ...draft, deleted_at: undefined };
    drafts.set(uuid, updated);
    return updated;
  },

  async delete(uuid: string): Promise<void> {
    const draft = drafts.get(uuid);
    if (!draft) throw new Error(`Draft ${uuid} not found`);
    
    const updated = { ...draft, deleted_at: getNow() };
    drafts.set(uuid, updated);
  },

  async hardDelete(uuid: string): Promise<void> {
    drafts.delete(uuid);
  }
};

export function resetMockBackend(): void {
  drafts.clear();
}
