import type { DraftAPI, DraftData } from '@dashtext/lib';

let nextId = 1;
const drafts: Map<number, DraftData> = new Map();

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
      id: nextId++,
      content: '',
      created_at: now,
      modified_at: now
    };
    drafts.set(draft.id, draft);
    return draft;
  },

  async get(id: number): Promise<DraftData | null> {
    return drafts.get(id) ?? null;
  },

  async save(id: number, content: string): Promise<DraftData> {
    const draft = drafts.get(id);
    if (!draft) throw new Error(`Draft ${id} not found`);

    const updated = { ...draft, content, modified_at: getNow() };
    drafts.set(id, updated);
    return updated;
  },

  async delete(id: number): Promise<void> {
    drafts.delete(id);
  }
};

export function resetMockBackend(): void {
  drafts.clear();
  nextId = 1;
}
