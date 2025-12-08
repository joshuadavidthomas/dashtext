import { eq, desc } from 'drizzle-orm';
import { getDb, drafts } from '$lib/db';
import type { DraftAPI, DraftData } from '@dashtext/lib';
import type { Draft as DrizzleDraft } from '$lib/db/schema';

/**
 * Convert Drizzle row (camelCase) to DraftData (snake_case)
 */
function toApiFormat(row: DrizzleDraft): DraftData {
  return {
    id: row.id,
    content: row.content,
    created_at: row.createdAt,
    modified_at: row.modifiedAt,
  };
}

/**
 * Get current ISO timestamp string
 */
function getNow(): string {
  return new Date().toISOString();
}

const tauriBackend: DraftAPI = {
  async list(): Promise<DraftData[]> {
    const db = await getDb();
    const rows = await db.select().from(drafts).orderBy(desc(drafts.modifiedAt));
    return rows.map(toApiFormat);
  },

  async create(): Promise<DraftData> {
    const db = await getDb();
    const now = getNow();
    const result = await db.insert(drafts).values({
      content: '',
      createdAt: now,
      modifiedAt: now,
    }).returning();
    return toApiFormat(result[0]);
  },

  async get(id: number): Promise<DraftData | null> {
    const db = await getDb();
    const rows = await db.select().from(drafts).where(eq(drafts.id, id));
    return rows.length > 0 ? toApiFormat(rows[0]) : null;
  },

  async save(id: number, content: string): Promise<DraftData> {
    const db = await getDb();
    const now = getNow();
    const result = await db.update(drafts)
      .set({ content, modifiedAt: now })
      .where(eq(drafts.id, id))
      .returning();
    return toApiFormat(result[0]);
  },

  async delete(id: number): Promise<void> {
    const db = await getDb();
    await db.delete(drafts).where(eq(drafts.id, id));
  },
};

export default tauriBackend;
