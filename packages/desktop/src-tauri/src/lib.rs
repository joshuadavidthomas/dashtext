mod db;
mod hotkey;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(db::plugin())
        .setup(|app| {
            // Initialize hotkey system
            let manager = hotkey::create_manager(app.handle().clone())?;
            if let Err(e) = manager.register() {
                tracing::error!("Failed to register global hotkey: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            updater::check_update,
            updater::can_auto_update,
            updater::download_and_install_update,
            updater::restart_app,
            updater::get_current_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
