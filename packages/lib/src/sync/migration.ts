import type { SyncEngine } from './SyncEngine';
import type { DbExecutor } from './storage/db-executor';

interface LegacyDraftRow {
  uuid: string;
  content: string;
  created_at: string;
  modified_at: string;
  deleted_at: string | null;
  archived: boolean;
  pinned: boolean;
}

/**
 * Check if migration from legacy draft table is needed.
 * 
 * Returns true if:
 * - automerge_doc_map table is empty (not yet migrated)
 * - AND draft table has rows (there are legacy drafts to migrate)
 * 
 * This is idempotent - returns false after first successful migration.
 */
export async function needsMigration(db: DbExecutor): Promise<boolean> {
  const rows = await db.select<{ count: number }>(
    `SELECT COUNT(*) as count FROM automerge_doc_map`
  );
  
  if (rows[0].count > 0) {
    return false;
  }
  
  const draftRows = await db.select<{ count: number }>(
    `SELECT COUNT(*) as count FROM draft`
  );
  
  return draftRows[0].count > 0;
}

/**
 * Migrate existing drafts from legacy draft table to Automerge.
 * 
 * This function is idempotent - safe to call multiple times.
 * It will skip migration if Automerge documents already exist.
 * 
 * Migration steps:
 * 1. Check if migration is needed
 * 2. Load all existing drafts from draft table
 * 3. Handle single-pin constraint (keep most recently modified pinned draft)
 * 4. For each draft:
 *    - Create new Automerge document
 *    - Set content using splice operations
 *    - Update metadata in root document
 * 5. Log completion
 */
export async function migrateExistingDrafts(
  syncEngine: SyncEngine,
  db: DbExecutor
): Promise<void> {
  if (!(await needsMigration(db))) {
    console.log('[Migration] Already migrated or no drafts to migrate');
    return;
  }

  console.log('[Migration] Starting migration of existing drafts');

  const legacyDrafts = await db.select<LegacyDraftRow>(
    `SELECT uuid, content, created_at, modified_at, deleted_at, archived, pinned
     FROM draft
     ORDER BY created_at ASC`
  );

  const pinnedDrafts = legacyDrafts.filter((d) => d.pinned);
  let pinnedUuid: string | null = null;
  
  if (pinnedDrafts.length > 1) {
    const mostRecent = pinnedDrafts.reduce((latest, current) => 
      current.modified_at > latest.modified_at ? current : latest
    );
    pinnedUuid = mostRecent.uuid;
    console.warn(
      `[Migration] Found ${pinnedDrafts.length} pinned drafts, ` +
      `keeping most recent: ${pinnedUuid}`
    );
  } else if (pinnedDrafts.length === 1) {
    pinnedUuid = pinnedDrafts[0].uuid;
  }

  for (const legacyDraft of legacyDrafts) {
    await migrateSingleDraft(syncEngine, legacyDraft, pinnedUuid);
  }

  console.log(`[Migration] Completed migration of ${legacyDrafts.length} drafts`);
}

async function migrateSingleDraft(
  syncEngine: SyncEngine,
  legacy: LegacyDraftRow,
  pinnedUuid: string | null
): Promise<void> {
  console.log(`[Migration] Migrating draft: ${legacy.uuid}`);

  const result = await syncEngine.createDraft();
  
  await syncEngine.setDraftContent(result.uuid, legacy.content);
  
  await syncEngine.updateDraftMetadata(result.uuid, {
    archived: legacy.archived,
    pinned: legacy.pinned && legacy.uuid === pinnedUuid,
    deletedAt: legacy.deleted_at
  });
}
