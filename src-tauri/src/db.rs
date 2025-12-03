use anyhow::Result;
use directories::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;

/// Get the ProjectDirs instance for dashtext
pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "dashtext")
        .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))
}

/// Get the database path using XDG directories
/// Linux: ~/.local/share/dashtext/dashtext.db
/// macOS: ~/Library/Application Support/dashtext/dashtext.db
/// Windows: C:\Users\<User>\AppData\Roaming\dashtext\dashtext.db
pub fn get_db_path() -> Result<PathBuf> {
    Ok(get_project_dirs()?.data_dir().join("dashtext.db"))
}

#[derive(Clone)]
pub struct Database(SqlitePool);

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.0
    }
}

pub async fn init_db() -> Result<SqlitePool> {
    let db_path = get_db_path()?;

    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Create connection options with create_if_missing
    let db_url = format!("sqlite://{}", db_path.display());
    let connect_options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Draft {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub modified_at: String,
}

impl Draft {
    /// Create a new empty draft
    pub async fn create(pool: &SqlitePool) -> Result<Draft> {
        let now = chrono_now();
        let result = sqlx::query_as::<_, Draft>(
            r#"
            INSERT INTO draft (content, created_at, modified_at)
            VALUES ('', ?1, ?1)
            RETURNING id, content, created_at, modified_at
            "#,
        )
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Find a draft by ID
    pub async fn find(pool: &SqlitePool, id: i64) -> Result<Option<Draft>> {
        let result = sqlx::query_as::<_, Draft>(
            "SELECT id, content, created_at, modified_at FROM draft WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// List all drafts, most recently modified first
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Draft>> {
        let results = sqlx::query_as::<_, Draft>(
            "SELECT id, content, created_at, modified_at FROM draft ORDER BY modified_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// Save this draft (update content and modified_at)
    pub async fn save(pool: &SqlitePool, id: i64, content: &str) -> Result<Draft> {
        let now = chrono_now();
        let result = sqlx::query_as::<_, Draft>(
            r#"
            UPDATE draft SET content = ?, modified_at = ?
            WHERE id = ?
            RETURNING id, content, created_at, modified_at
            "#,
        )
        .bind(content)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Delete a draft by ID
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM draft WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Get current time as ISO 8601 string
fn chrono_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Format as ISO 8601 (simplified - just use Unix timestamp for now, or add chrono later)
    format!("{}", now)
}
