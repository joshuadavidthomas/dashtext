import type { DraftAPI, DraftData } from '../types';

/**
 * In-memory mock backend for browser testing (non-Tauri environments)
 */
class MockBackend implements DraftAPI {
  private drafts: Map<number, DraftData> = new Map();
  private nextId = 1;

  async list(): Promise<DraftData[]> {
    return Array.from(this.drafts.values()).sort(
      (a, b) => new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime()
    );
  }

  async create(): Promise<DraftData> {
    const now = new Date().toISOString();
    const draft: DraftData = {
      id: this.nextId++,
      content: '',
      created_at: now,
      modified_at: now,
    };
    this.drafts.set(draft.id, draft);
    return draft;
  }

  async get(id: number): Promise<DraftData | null> {
    return this.drafts.get(id) ?? null;
  }

  async save(id: number, content: string): Promise<DraftData> {
    const draft = this.drafts.get(id);
    if (!draft) {
      throw new Error(`Draft ${id} not found`);
    }
    draft.content = content;
    draft.modified_at = new Date().toISOString();
    return draft;
  }

  async delete(id: number): Promise<void> {
    this.drafts.delete(id);
  }
}

export const mockBackend: DraftAPI = new MockBackend();
