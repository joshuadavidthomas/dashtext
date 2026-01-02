-- Automerge chunk storage for repo persistence
CREATE TABLE `automerge_chunk` (
	`doc_id` text NOT NULL,
	`chunk_type` text NOT NULL,
	`chunk_id` text NOT NULL,
	`bytes` blob NOT NULL,
	`created_at` text NOT NULL,
	PRIMARY KEY (`doc_id`, `chunk_type`, `chunk_id`)
);
--> statement-breakpoint
CREATE INDEX `idx_chunk_doc_id` ON `automerge_chunk` (`doc_id`);
--> statement-breakpoint
CREATE INDEX `idx_chunk_doc_type` ON `automerge_chunk` (`doc_id`, `chunk_type`);
--> statement-breakpoint
-- Draft UUID to Automerge document ID mapping
CREATE TABLE `automerge_doc_map` (
	`draft_uuid` text PRIMARY KEY NOT NULL,
	`doc_id` text NOT NULL UNIQUE,
	`created_at` text NOT NULL
);
--> statement-breakpoint
-- Sync configuration (singleton row)
CREATE TABLE `sync_state` (
	`id` integer PRIMARY KEY CHECK (id = 1),
	`sync_enabled` integer DEFAULT 0 NOT NULL,
	`space_id` text,
	`device_id` text,
	`auth_token` text,
	`server_url` text,
	`root_doc_id` text,
	`last_connected_at` text,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
