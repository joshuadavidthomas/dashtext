mod conf;
mod db;
mod hotkey;
mod updater;

use tauri::Manager;

/// Register capture shortcut hotkey
#[tauri::command]
async fn register_capture_shortcut(
    shortcut: String,
    state: tauri::State<'_, conf::SettingsState>,
) -> Result<(), String> {
    state.register_capture_shortcut(&shortcut).await
}

/// Unregister capture shortcut hotkey
#[tauri::command]
async fn unregister_capture_shortcut(
    state: tauri::State<'_, conf::SettingsState>,
) -> Result<(), String> {
    state.unregister_capture_shortcut().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(db::plugin())
        .setup(|app| {
            // Initialize settings state
            let settings_state = conf::SettingsState::new(app.handle().clone());
            app.manage(settings_state);

            // Note: Hotkey will be registered by frontend after loading settings from database
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            updater::check_update,
            updater::can_auto_update,
            updater::download_and_install_update,
            updater::restart_app,
            updater::get_current_version,
            register_capture_shortcut,
            unregister_capture_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
