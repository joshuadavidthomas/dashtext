import { sqliteTable, integer, text, index } from 'drizzle-orm/sqlite-core';

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