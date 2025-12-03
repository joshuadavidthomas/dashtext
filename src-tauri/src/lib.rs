mod db;

use db::{Database, Draft};
use tauri::Manager;

#[tauri::command]
async fn draft_create(db: tauri::State<'_, Database>) -> Result<Draft, String> {
    Draft::create(db.pool()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_get(db: tauri::State<'_, Database>, id: i64) -> Result<Option<Draft>, String> {
    Draft::find(db.pool(), id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_list(db: tauri::State<'_, Database>) -> Result<Vec<Draft>, String> {
    Draft::list(db.pool()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_save(
    db: tauri::State<'_, Database>,
    id: i64,
    content: String,
) -> Result<Draft, String> {
    Draft::save(db.pool(), id, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn draft_delete(db: tauri::State<'_, Database>, id: i64) -> Result<(), String> {
    Draft::delete(db.pool(), id)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database asynchronously
            // Uses XDG directories via the `directories` crate
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match db::init_db().await {
                    Ok(pool) => {
                        app_handle.manage(Database::new(pool));
                        match db::get_db_path() {
                            Ok(path) => println!("Database initialized at: {}", path.display()),
                            Err(_) => println!("Database initialized"),
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize database: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            draft_create,
            draft_get,
            draft_list,
            draft_save,
            draft_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
