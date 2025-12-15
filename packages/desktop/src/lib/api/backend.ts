import { eq, desc, isNull } from 'drizzle-orm';
import { getDb, drafts } from '$lib/db';
import type { DraftAPI, DraftData, DraftInternal } from '@dashtext/lib';
import { generateUUID } from '@dashtext/lib';
import type { Draft } from '$lib/db/schema';

/**
 * Convert Drizzle row (camelCase) to public DraftData (snake_case, UUID only)
 */
function toApiFormat(row: Draft): DraftData {
  return {
    uuid: row.uuid,
    content: row.content,
    created_at: row.createdAt,
    modified_at: row.modifiedAt,
    ...(row.deletedAt && { deleted_at: row.deletedAt }),
    ...(row.archived && { archived: row.archived }),
    ...(row.pinned && { pinned: row.pinned }),
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
    const rows = await db.select()
      .from(drafts)
      .where(isNull(drafts.deletedAt))
      .orderBy(desc(drafts.modifiedAt));
    return rows.map(toApiFormat);
  },

  async create(): Promise<DraftData> {
    const db = await getDb();
    const now = getNow();
    const result = await db.insert(drafts).values({
      uuid: generateUUID(),
      content: '',
      createdAt: now,
      modifiedAt: now,
      archived: false,
      pinned: false,
    }).returning();
    return toApiFormat(result[0]);
  },

  async get(uuid: string): Promise<DraftData | null> {
    const db = await getDb();
    const rows = await db.select()
      .from(drafts)
      .where(eq(drafts.uuid, uuid));
    return rows.length > 0 ? toApiFormat(rows[0]) : null;
  },

  async save(uuid: string, content: string): Promise<DraftData> {
    const db = await getDb();
    const now = getNow();
    const result = await db.update(drafts)
      .set({ content, modifiedAt: now })
      .where(eq(drafts.uuid, uuid))
      .returning();
    return toApiFormat(result[0]);
  },

  async archive(uuid: string): Promise<DraftData> {
    const db = await getDb();
    const result = await db.update(drafts)
      .set({ archived: true })
      .where(eq(drafts.uuid, uuid))
      .returning();
    return toApiFormat(result[0]);
  },

  async unarchive(uuid: string): Promise<DraftData> {
    const db = await getDb();
    const result = await db.update(drafts)
      .set({ archived: false })
      .where(eq(drafts.uuid, uuid))
      .returning();
    return toApiFormat(result[0]);
  },

  async pin(uuid: string): Promise<DraftData> {
    const db = await getDb();
    
    // Unpin all other drafts first (single pin constraint)
    await db.update(drafts)
      .set({ pinned: false })
      .where(eq(drafts.pinned, true));
    
    // Pin the requested draft
    const result = await db.update(drafts)
      .set({ pinned: true })
      .where(eq(drafts.uuid, uuid))
      .returning();
    return toApiFormat(result[0]);
  },

  async unpin(uuid: string): Promise<DraftData> {
    const db = await getDb();
    const result = await db.update(drafts)
      .set({ pinned: false })
      .where(eq(drafts.uuid, uuid))
      .returning();
    return toApiFormat(result[0]);
  },

  async restore(uuid: string): Promise<DraftData> {
    const db = await getDb();
    const result = await db.update(drafts)
      .set({ deletedAt: null })
      .where(eq(drafts.uuid, uuid))
      .returning();
    return toApiFormat(result[0]);
  },

  async delete(uuid: string): Promise<void> {
    const db = await getDb();
    const now = getNow();
    await db.update(drafts)
      .set({ deletedAt: now })
      .where(eq(drafts.uuid, uuid));
  },

  async hardDelete(uuid: string): Promise<void> {
    const db = await getDb();
    await db.delete(drafts).where(eq(drafts.uuid, uuid));
  },
};

export default tauriBackend;
