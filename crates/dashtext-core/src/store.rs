use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use rusqlite::Connection;
use rusqlite::ErrorCode;
use rusqlite::OptionalExtension as _;
use rusqlite::Row;
use rusqlite::TransactionBehavior;
use rusqlite::params;

use crate::draft::Draft;
use crate::draft::DraftId;
use crate::draft::Folder;
use crate::draft::Timestamp;
use crate::query::Scope;
use crate::query::Sort;
use crate::workspace::Workspace;
use crate::workspace::WorkspaceId;
use crate::workspace::WorkspaceView;

pub type Result<T, E = StoreError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error("draft {0} does not exist")]
    DraftNotFound(DraftId),
    #[error(
        "the library was created by a newer version of Dashtext (schema {found}, supported {supported})"
    )]
    NewerSchema { found: i64, supported: i64 },
    #[error("invalid data in the library: {0}")]
    Corrupt(String),
}

/// Ordered schema migrations. Each entry moves `user_version` from its index
/// to its index plus one. Never edit a released migration; append a new one.
const MIGRATIONS: &[&str] = &[
    // 1: drafts, workspaces and settings.
    "
    CREATE TABLE drafts (
        id TEXT PRIMARY KEY NOT NULL,
        content TEXT NOT NULL,
        folder TEXT NOT NULL DEFAULT 'inbox' CHECK (folder IN ('inbox', 'archive', 'trash')),
        flagged INTEGER NOT NULL DEFAULT 0 CHECK (flagged IN (0, 1)),
        created_at INTEGER NOT NULL,
        modified_at INTEGER NOT NULL,
        accessed_at INTEGER NOT NULL,
        trashed_at INTEGER
    ) STRICT;

    CREATE INDEX drafts_by_folder ON drafts (folder, modified_at);

    CREATE TABLE workspaces (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        position INTEGER NOT NULL,
        view TEXT NOT NULL DEFAULT '{}',
        created_at INTEGER NOT NULL,
        modified_at INTEGER NOT NULL
    ) STRICT;

    CREATE TABLE settings (
        key TEXT PRIMARY KEY NOT NULL,
        value TEXT NOT NULL
    ) STRICT;
    ",
];

/// How long to wait for another process's write before failing.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

const DRAFT_COLUMNS: &str =
    "id, content, folder, flagged, created_at, modified_at, accessed_at, trashed_at";

/// Draft counts for each [`Scope`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeCounts {
    inbox: usize,
    flagged: usize,
    archive: usize,
    all: usize,
    trash: usize,
}

impl ScopeCounts {
    #[must_use]
    pub fn get(&self, scope: Scope) -> usize {
        match scope {
            Scope::Inbox => self.inbox,
            Scope::Flagged => self.flagged,
            Scope::Archive => self.archive,
            Scope::All => self.all,
            Scope::Trash => self.trash,
        }
    }
}

/// The draft library, persisted in SQLite.
///
/// Several processes may open the same library (the app and `dashtext new`,
/// for example); the database runs in WAL mode with a busy timeout so short
/// concurrent writes wait for each other instead of failing.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens the library at `path`, creating and migrating it as needed.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Before anything that takes a lock: another process (the app, or
        // `dashtext new`) may be creating or migrating the same library.
        conn.busy_timeout(BUSY_TIMEOUT)?;
        enable_wal(&conn)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Self::init(conn)
    }

    /// Opens a private, empty library that lives only in memory.
    #[cfg(test)]
    fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let supported = i64::try_from(MIGRATIONS.len()).unwrap_or(i64::MAX);
        // Take the write lock before reading the version, so two processes
        // opening a new library cannot both run the same migration.
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let found: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if found > supported {
            return Err(StoreError::NewerSchema { found, supported });
        }

        for (version, sql) in (1_i64..).zip(MIGRATIONS).skip_while(|(v, _)| *v <= found) {
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Creates a draft with `content` in `folder`.
    pub fn create_draft(&self, content: &str, folder: Folder, flagged: bool) -> Result<Draft> {
        let now = Timestamp::now();
        let draft = Draft {
            id: DraftId::new(),
            content: content.to_owned(),
            folder,
            flagged,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            trashed_at: (folder == Folder::Trash).then_some(now),
        };
        self.conn.execute(
            &format!(
                "INSERT INTO drafts ({DRAFT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            ),
            params![
                draft.id.to_string(),
                draft.content,
                draft.folder.as_str(),
                draft.flagged,
                draft.created_at.as_millis(),
                draft.modified_at.as_millis(),
                draft.accessed_at.as_millis(),
                draft.trashed_at.map(Timestamp::as_millis),
            ],
        )?;
        Ok(draft)
    }

    pub fn draft(&self, id: DraftId) -> Result<Option<Draft>> {
        self.conn
            .query_row(
                &format!("SELECT {DRAFT_COLUMNS} FROM drafts WHERE id = ?1"),
                [id.to_string()],
                draft_from_row,
            )
            .optional()?
            .transpose()
    }

    fn existing_draft(&self, id: DraftId) -> Result<Draft> {
        self.draft(id)?.ok_or(StoreError::DraftNotFound(id))
    }

    /// Returns the drafts in `scope`, ordered by `sort`.
    pub fn drafts(&self, scope: Scope, sort: Sort) -> Result<Vec<Draft>> {
        let filter = match scope {
            Scope::Inbox => "folder = 'inbox'",
            Scope::Flagged => "flagged = 1 AND folder != 'trash'",
            Scope::Archive => "folder = 'archive'",
            Scope::All => "folder != 'trash'",
            Scope::Trash => "folder = 'trash'",
        };
        let mut statement = self.conn.prepare(&format!(
            "SELECT {DRAFT_COLUMNS} FROM drafts WHERE {filter} ORDER BY {}",
            sort.order_by()
        ))?;
        let rows = statement.query_map([], draft_from_row)?;
        rows.map(|row| row?).collect()
    }

    pub fn counts(&self) -> Result<ScopeCounts> {
        Ok(self.conn.query_row(
            "SELECT
                COUNT(*) FILTER (WHERE folder = 'inbox'),
                COUNT(*) FILTER (WHERE flagged = 1 AND folder != 'trash'),
                COUNT(*) FILTER (WHERE folder = 'archive'),
                COUNT(*) FILTER (WHERE folder != 'trash'),
                COUNT(*) FILTER (WHERE folder = 'trash')
            FROM drafts",
            [],
            |row| {
                Ok(ScopeCounts {
                    inbox: count_at(row, 0)?,
                    flagged: count_at(row, 1)?,
                    archive: count_at(row, 2)?,
                    all: count_at(row, 3)?,
                    trash: count_at(row, 4)?,
                })
            },
        )?)
    }

    /// Replaces a draft's content. The modification time only moves when the
    /// content actually changes.
    pub fn update_content(&self, id: DraftId, content: &str) -> Result<Draft> {
        let now = Timestamp::now();
        self.conn.execute(
            "UPDATE drafts SET content = ?2, modified_at = ?3 WHERE id = ?1 AND content != ?2",
            params![id.to_string(), content, now.as_millis()],
        )?;
        self.existing_draft(id)
    }

    pub fn set_flagged(&self, id: DraftId, flagged: bool) -> Result<Draft> {
        self.conn.execute(
            "UPDATE drafts SET flagged = ?2 WHERE id = ?1",
            params![id.to_string(), flagged],
        )?;
        self.existing_draft(id)
    }

    /// Moves a draft to `folder`, recording when it entered the trash.
    pub fn move_to(&self, id: DraftId, folder: Folder) -> Result<Draft> {
        let trashed_at = (folder == Folder::Trash).then(|| Timestamp::now().as_millis());
        self.conn.execute(
            "UPDATE drafts
             SET folder = ?2,
                 trashed_at = CASE WHEN folder = ?2 THEN trashed_at ELSE ?3 END
             WHERE id = ?1",
            params![id.to_string(), folder.as_str(), trashed_at],
        )?;
        self.existing_draft(id)
    }

    /// Records that the draft was opened.
    pub fn mark_accessed(&self, id: DraftId) -> Result<()> {
        self.conn.execute(
            "UPDATE drafts SET accessed_at = ?2 WHERE id = ?1",
            params![id.to_string(), Timestamp::now().as_millis()],
        )?;
        Ok(())
    }

    /// Permanently deletes a draft.
    pub fn delete(&self, id: DraftId) -> Result<()> {
        self.conn
            .execute("DELETE FROM drafts WHERE id = ?1", [id.to_string()])?;
        Ok(())
    }

    /// Permanently deletes every draft in the trash.
    pub fn empty_trash(&self) -> Result<usize> {
        Ok(self
            .conn
            .execute("DELETE FROM drafts WHERE folder = 'trash'", [])?)
    }

    /// Permanently deletes drafts that entered the trash before `cutoff`.
    pub fn purge_trash(&self, cutoff: Timestamp) -> Result<usize> {
        Ok(self.conn.execute(
            "DELETE FROM drafts WHERE folder = 'trash' AND trashed_at < ?1",
            [cutoff.as_millis()],
        )?)
    }

    /// Returns the first workspace, creating the default one on first use.
    pub fn default_workspace(&self) -> Result<Workspace> {
        let existing = self
            .conn
            .query_row(
                "SELECT id, name, view FROM workspaces ORDER BY position, created_at LIMIT 1",
                [],
                workspace_from_row,
            )
            .optional()?
            .transpose()?;
        if let Some(workspace) = existing {
            return Ok(workspace);
        }

        let workspace = Workspace::new("Drafts");
        self.save_workspace(&workspace)?;
        Ok(workspace)
    }

    pub fn save_workspace(&self, workspace: &Workspace) -> Result<()> {
        let now = Timestamp::now().as_millis();
        let view = serde_json::to_string(workspace.view())
            .map_err(|error| StoreError::Corrupt(error.to_string()))?;
        self.conn.execute(
            "INSERT INTO workspaces (id, name, position, view, created_at, modified_at)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(position) + 1, 0) FROM workspaces), ?3, ?4, ?4)
             ON CONFLICT (id) DO UPDATE
             SET name = excluded.name, view = excluded.view, modified_at = excluded.modified_at",
            params![workspace.id().to_string(), workspace.name(), view, now],
        )?;
        Ok(())
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }

    pub fn remove_setting(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM settings WHERE key = ?1", [key])?;
        Ok(())
    }
}

fn count_at(row: &Row<'_>, ix: usize) -> rusqlite::Result<usize> {
    let count: i64 = row.get(ix)?;
    Ok(usize::try_from(count).unwrap_or(0))
}

/// Maps a row to a draft. The outer `Result` carries SQLite errors, the inner
/// one rejects values SQLite accepted but the domain does not.
fn draft_from_row(row: &Row<'_>) -> rusqlite::Result<Result<Draft>> {
    let id: String = row.get(0)?;
    let folder: String = row.get(2)?;
    let content = row.get(1)?;
    let flagged = row.get(3)?;
    let created_at = Timestamp::from_millis(row.get(4)?);
    let modified_at = Timestamp::from_millis(row.get(5)?);
    let accessed_at = Timestamp::from_millis(row.get(6)?);
    let trashed_at = row.get::<_, Option<i64>>(7)?.map(Timestamp::from_millis);

    Ok((|| {
        Ok(Draft {
            id: id
                .parse()
                .map_err(|error| StoreError::Corrupt(format!("draft id `{id}`: {error}")))?,
            content,
            folder: folder
                .parse()
                .map_err(|error| StoreError::Corrupt(format!("{error}")))?,
            flagged,
            created_at,
            modified_at,
            accessed_at,
            trashed_at,
        })
    })())
}

fn workspace_from_row(row: &Row<'_>) -> rusqlite::Result<Result<Workspace>> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let view: String = row.get(2)?;

    Ok((|| {
        let id: WorkspaceId = id
            .parse()
            .map_err(|error| StoreError::Corrupt(format!("workspace id `{id}`: {error}")))?;
        // A view written by a newer version may not parse; fall back to the
        // defaults rather than locking the user out of their library.
        let view: WorkspaceView = serde_json::from_str(&view).unwrap_or_default();
        Ok(Workspace::from_parts(id, name, view))
    })())
}

/// Switches `conn` to write-ahead logging. Changing the journal mode needs an exclusive lock,
/// and SQLite reports `SQLITE_BUSY` at once instead of waiting in the busy handler, so this
/// retries while another process opens the same library.
fn enable_wal(conn: &Connection) -> Result<()> {
    let deadline = Instant::now() + BUSY_TIMEOUT;
    loop {
        match conn.pragma_update(None, "journal_mode", "WAL") {
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == ErrorCode::DatabaseBusy && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            result => return Ok(result?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::SortDirection;
    use crate::query::SortKey;

    fn store() -> Store {
        Store::open_in_memory().expect("open in-memory store")
    }

    #[test]
    fn created_drafts_land_in_their_folder() {
        let store = store();
        let draft = store
            .create_draft("Hello\nworld", Folder::Inbox, false)
            .expect("store operation");

        assert_eq!(
            store.draft(draft.id()).expect("store operation"),
            Some(draft.clone())
        );
        assert_eq!(
            store
                .drafts(Scope::Inbox, Sort::default())
                .expect("store operation"),
            vec![draft]
        );
        assert!(
            store
                .drafts(Scope::Archive, Sort::default())
                .expect("store operation")
                .is_empty()
        );
    }

    #[test]
    fn scopes_follow_folders_and_flags() {
        let store = store();
        let inbox = store
            .create_draft("inbox", Folder::Inbox, true)
            .expect("store operation");
        let archived = store
            .create_draft("archived", Folder::Archive, false)
            .expect("store operation");
        let trashed = store
            .create_draft("trashed", Folder::Trash, true)
            .expect("store operation");

        let ids = |scope| -> Vec<DraftId> {
            store
                .drafts(
                    scope,
                    Sort {
                        key: SortKey::Created,
                        direction: SortDirection::Ascending,
                    },
                )
                .expect("store operation")
                .iter()
                .map(Draft::id)
                .collect()
        };
        assert_eq!(ids(Scope::Inbox), vec![inbox.id()]);
        assert_eq!(ids(Scope::Flagged), vec![inbox.id()]);
        assert_eq!(ids(Scope::Archive), vec![archived.id()]);
        assert_eq!(ids(Scope::All), vec![inbox.id(), archived.id()]);
        assert_eq!(ids(Scope::Trash), vec![trashed.id()]);

        let counts = store.counts().expect("store operation");
        for scope in Scope::ALL {
            assert_eq!(counts.get(scope), ids(scope).len(), "{scope:?}");
        }
    }

    #[test]
    fn scope_contains_agrees_with_queries() {
        let store = store();
        store
            .create_draft("a", Folder::Inbox, true)
            .expect("store operation");
        store
            .create_draft("b", Folder::Archive, true)
            .expect("store operation");
        store
            .create_draft("c", Folder::Trash, false)
            .expect("store operation");
        let everything: Vec<Draft> = [Scope::All, Scope::Trash]
            .into_iter()
            .map(|scope| store.drafts(scope, Sort::default()))
            .collect::<Result<Vec<_>>>()
            .expect("list drafts")
            .concat();

        for scope in Scope::ALL {
            let expected = store
                .drafts(scope, Sort::default())
                .expect("store operation")
                .len();
            let actual = everything.iter().filter(|d| scope.contains(d)).count();
            assert_eq!(actual, expected, "{scope:?}");
        }
    }

    #[test]
    fn update_content_only_moves_modified_time_on_change() {
        let store = store();
        let draft = store
            .create_draft("same", Folder::Inbox, false)
            .expect("store operation");
        store
            .conn
            .execute("UPDATE drafts SET modified_at = 1", [])
            .expect("store operation");

        let unchanged = store
            .update_content(draft.id(), "same")
            .expect("store operation");
        assert_eq!(unchanged.modified_at(), Timestamp::from_millis(1));

        let changed = store
            .update_content(draft.id(), "different")
            .expect("store operation");
        assert_eq!(changed.content(), "different");
        assert!(changed.modified_at() > Timestamp::from_millis(1));
    }

    #[test]
    fn moving_to_trash_records_when() {
        let store = store();
        let draft = store
            .create_draft("x", Folder::Inbox, false)
            .expect("store operation");

        let trashed = store
            .move_to(draft.id(), Folder::Trash)
            .expect("store operation");
        assert_eq!(trashed.folder(), Folder::Trash);
        assert!(trashed.trashed_at.is_some());

        let restored = store
            .move_to(draft.id(), Folder::Inbox)
            .expect("store operation");
        assert_eq!(restored.folder(), Folder::Inbox);
        assert_eq!(restored.trashed_at, None);
    }

    #[test]
    fn purge_trash_removes_only_old_trash() {
        let store = store();
        let old = store
            .create_draft("old", Folder::Trash, false)
            .expect("store operation");
        let recent = store
            .create_draft("recent", Folder::Trash, false)
            .expect("store operation");
        let kept = store
            .create_draft("kept", Folder::Inbox, false)
            .expect("store operation");
        store
            .conn
            .execute(
                "UPDATE drafts SET trashed_at = 10 WHERE id = ?1",
                [old.id().to_string()],
            )
            .expect("store operation");

        assert_eq!(
            store
                .purge_trash(Timestamp::from_millis(20))
                .expect("store operation"),
            1
        );
        assert_eq!(store.draft(old.id()).expect("store operation"), None);
        assert!(store.draft(recent.id()).expect("store operation").is_some());
        assert!(store.draft(kept.id()).expect("store operation").is_some());

        assert_eq!(store.empty_trash().expect("store operation"), 1);
        assert_eq!(
            store.counts().expect("store operation").get(Scope::Trash),
            0
        );
    }

    #[test]
    fn missing_drafts_are_reported() {
        let store = store();
        let id = DraftId::new();
        assert!(matches!(
            store.set_flagged(id, true),
            Err(StoreError::DraftNotFound(missing)) if missing == id
        ));
    }

    #[test]
    fn sort_orders_by_requested_timestamp() {
        let store = store();
        let first = store
            .create_draft("first", Folder::Inbox, false)
            .expect("store operation");
        let second = store
            .create_draft("second", Folder::Inbox, false)
            .expect("store operation");
        store
            .conn
            .execute(
                "UPDATE drafts SET modified_at = 100 WHERE id = ?1",
                [second.id().to_string()],
            )
            .expect("store operation");
        store
            .conn
            .execute(
                "UPDATE drafts SET modified_at = 200 WHERE id = ?1",
                [first.id().to_string()],
            )
            .expect("store operation");

        let by_modified = store
            .drafts(Scope::Inbox, Sort::default())
            .expect("store operation");
        assert_eq!(by_modified[0].id(), first.id());
        let by_created = store
            .drafts(
                Scope::Inbox,
                Sort {
                    key: SortKey::Created,
                    direction: SortDirection::Descending,
                },
            )
            .expect("store operation");
        assert_eq!(by_created[0].id(), second.id());
    }

    #[test]
    fn sort_compare_matches_query_order() {
        let store = store();
        // Repeated timestamps exercise the id tie-break.
        for (content, modified, accessed) in
            [("a", 100, 9), ("b", 101, 8), ("c", 100, 8), ("d", 101, 7)]
        {
            let draft = store
                .create_draft(content, Folder::Inbox, false)
                .expect("store operation");
            store
                .conn
                .execute(
                    "UPDATE drafts SET modified_at = ?2, accessed_at = ?3 WHERE id = ?1",
                    params![draft.id().to_string(), modified, accessed],
                )
                .expect("store operation");
        }
        for key in [SortKey::Created, SortKey::Modified, SortKey::Accessed] {
            for direction in [SortDirection::Ascending, SortDirection::Descending] {
                let sort = Sort { key, direction };
                let queried = store.drafts(Scope::Inbox, sort).expect("store operation");
                let mut sorted = queried.clone();
                sorted.sort_by(|a, b| sort.compare(a, b));
                assert_eq!(sorted, queried, "{sort:?}");
            }
        }
    }

    #[test]
    fn default_workspace_is_created_once_and_saved() {
        let store = store();
        let mut workspace = store.default_workspace().expect("store operation");
        assert_eq!(workspace.scope(), Scope::Inbox);

        workspace.set_scope(Scope::Archive);
        store.save_workspace(&workspace).expect("store operation");
        assert_eq!(
            store.default_workspace().expect("store operation"),
            workspace
        );
    }

    #[test]
    fn settings_round_trip() {
        let store = store();
        assert_eq!(
            store.setting("capture.buffer").expect("store operation"),
            None
        );
        store
            .set_setting("capture.buffer", "one")
            .expect("store operation");
        store
            .set_setting("capture.buffer", "two")
            .expect("store operation");
        assert_eq!(
            store
                .setting("capture.buffer")
                .expect("store operation")
                .as_deref(),
            Some("two")
        );
        store
            .remove_setting("capture.buffer")
            .expect("store operation");
        assert_eq!(
            store.setting("capture.buffer").expect("store operation"),
            None
        );
    }

    #[test]
    fn reopening_a_file_keeps_drafts() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("library.db");
        let id = Store::open(&path)
            .expect("store operation")
            .create_draft("persisted", Folder::Inbox, false)
            .expect("store operation")
            .id();

        let reopened = Store::open(&path).expect("store operation");
        assert_eq!(
            reopened
                .draft(id)
                .expect("store operation")
                .map(|d| d.content().to_owned()),
            Some("persisted".to_owned())
        );
    }

    #[test]
    fn concurrent_first_opens_migrate_once() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("library.db");
        let openers: Vec<_> = (0..4)
            .map(|_| {
                let path = path.clone();
                std::thread::spawn(move || Store::open(&path).map(|_| ()))
            })
            .collect();
        for opener in openers {
            opener
                .join()
                .expect("opener thread")
                .expect("every process opens the library");
        }
    }

    #[test]
    fn newer_schema_is_refused() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("library.db");
        Connection::open(&path)
            .expect("store operation")
            .pragma_update(None, "user_version", 999)
            .expect("store operation");

        assert!(matches!(
            Store::open(&path),
            Err(StoreError::NewerSchema { found: 999, .. })
        ));
    }
}
