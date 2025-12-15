DROP INDEX IF EXISTS `idx_draft_uuid`;
DROP INDEX IF EXISTS `idx_draft_deleted_at`;
DROP INDEX IF EXISTS `idx_draft_archived`;
DROP INDEX IF EXISTS `idx_draft_pinned`;--> statement-breakpoint
-- Note: We keep the columns for safety during transition
-- Later migration can remove them if needed