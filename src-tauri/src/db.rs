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
