import { sqliteTable, integer, text, index } from 'drizzle-orm/sqlite-core';

export const drafts = sqliteTable('draft', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  content: text('content').notNull().default(''),
  createdAt: text('created_at').notNull(),
  modifiedAt: text('modified_at').notNull(),
}, (table) => [
  index('idx_draft_modified_at').on(table.modifiedAt),
]);

export type Draft = typeof drafts.$inferSelect;
export type NewDraft = typeof drafts.$inferInsert;
