-- Create draft table
CREATE TABLE IF NOT EXISTS draft (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

-- Index on modified_at for "most recent" queries
CREATE INDEX IF NOT EXISTS idx_draft_modified_at
ON draft(modified_at DESC);
