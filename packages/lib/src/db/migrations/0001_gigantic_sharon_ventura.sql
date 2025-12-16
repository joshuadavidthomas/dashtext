ALTER TABLE `draft` ADD `uuid` text NOT NULL DEFAULT '';--> statement-breakpoint
ALTER TABLE `draft` ADD `deleted_at` text;--> statement-breakpoint
ALTER TABLE `draft` ADD `archived` integer DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE `draft` ADD `pinned` integer DEFAULT false NOT NULL;--> statement-breakpoint
UPDATE `draft` SET `uuid` = lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6))) WHERE `uuid` = '';--> statement-breakpoint
CREATE UNIQUE INDEX `idx_draft_uuid` ON `draft` (`uuid`);--> statement-breakpoint
CREATE INDEX `idx_draft_deleted_at` ON `draft` (`deleted_at`);--> statement-breakpoint
CREATE INDEX `idx_draft_archived` ON `draft` (`archived`);--> statement-breakpoint
CREATE INDEX `idx_draft_pinned` ON `draft` (`pinned`);