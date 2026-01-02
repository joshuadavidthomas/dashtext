import { sqliteTable, integer, text, index, blob, primaryKey } from 'drizzle-orm/sqlite-core';

export const drafts = sqliteTable('draft', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  uuid: text('uuid').notNull().unique(),
  content: text('content').notNull().default(''),
  createdAt: text('created_at').notNull(),
  modifiedAt: text('modified_at').notNull(),
  deletedAt: text('deleted_at'),
  archived: integer('archived', { mode: 'boolean' }).default(false).notNull(),
  pinned: integer('pinned', { mode: 'boolean' }).default(false).notNull(),
}, (table) => [
  index('idx_draft_modified_at').on(table.modifiedAt),
  index('idx_draft_uuid').on(table.uuid),
  index('idx_draft_deleted_at').on(table.deletedAt),
  index('idx_draft_archived').on(table.archived),
  index('idx_draft_pinned').on(table.pinned),
]);

export type Draft = typeof drafts.$inferSelect;
export type NewDraft = typeof drafts.$inferInsert;

export const settings = sqliteTable('settings', {
  id: integer('id').primaryKey().$default(() => 1),
  captureShortcut: text('capture_shortcut'),
  createdAt: text('created_at').notNull(),
  updatedAt: text('updated_at').notNull(),
});

export type Settings = typeof settings.$inferSelect;
export type NewSettings = typeof settings.$inferInsert;

// Automerge sync tables

export const automergeChunk = sqliteTable('automerge_chunk', {
  docId: text('doc_id').notNull(),
  chunkType: text('chunk_type').notNull(),
  chunkId: text('chunk_id').notNull(),
  bytes: blob('bytes', { mode: 'buffer' }).notNull(),
  createdAt: text('created_at').notNull(),
}, (table) => [
  primaryKey({ columns: [table.docId, table.chunkType, table.chunkId] }),
  index('idx_chunk_doc_id').on(table.docId),
  index('idx_chunk_doc_type').on(table.docId, table.chunkType),
]);

export type AutomergeChunk = typeof automergeChunk.$inferSelect;
export type NewAutomergeChunk = typeof automergeChunk.$inferInsert;

export const automergeDocMap = sqliteTable('automerge_doc_map', {
  draftUuid: text('draft_uuid').primaryKey(),
  docId: text('doc_id').notNull().unique(),
  createdAt: text('created_at').notNull(),
});

export type AutomergeDocMap = typeof automergeDocMap.$inferSelect;
export type NewAutomergeDocMap = typeof automergeDocMap.$inferInsert;

export const syncState = sqliteTable('sync_state', {
  id: integer('id').primaryKey(),
  syncEnabled: integer('sync_enabled', { mode: 'boolean' }).default(false).notNull(),
  spaceId: text('space_id'),
  deviceId: text('device_id'),
  authToken: text('auth_token'),
  serverUrl: text('server_url'),
  rootDocId: text('root_doc_id'),
  lastConnectedAt: text('last_connected_at'),
  createdAt: text('created_at').notNull(),
  updatedAt: text('updated_at').notNull(),
});

export type SyncState = typeof syncState.$inferSelect;
export type NewSyncState = typeof syncState.$inferInsert;