//! Database configuration and migrations for dashtext.
//!
//! # Adding a new migration
//! 1. Update the Drizzle schema in `src/lib/db/schema.ts`
//! 2. Run `bun drizzle-kit generate`
//! 3. Add the generated SQL file to `get_migrations()` below
//! 4. Increment the version number

use tauri::{plugin::TauriPlugin, Runtime};
use tauri_plugin_sql::{Migration, MigrationKind, PluginConfig};

/// Database connection URL
pub const DB_URL: &str = "sqlite:dashtext.db";

/// Returns the configured SQL plugin with migrations.
pub fn plugin<R: Runtime>() -> TauriPlugin<R, Option<PluginConfig>> {
    tauri_plugin_sql::Builder::default()
        .add_migrations(DB_URL, get_migrations())
        .build()
}

/// Returns all database migrations in order.
fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_draft_table",
            sql: include_str!("../../src/lib/db/migrations/0000_clean_blockbuster.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
