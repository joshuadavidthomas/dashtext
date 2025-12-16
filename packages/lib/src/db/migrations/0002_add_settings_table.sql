CREATE TABLE `settings` (
	`id` integer PRIMARY KEY CHECK (id = 1),
	`capture_shortcut` text,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
