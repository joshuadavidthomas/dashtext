mod db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "create_draft_table",
        sql: include_str!("../../src/lib/db/migrations/0000_clean_blockbuster.sql"),
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(db::plugin())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
