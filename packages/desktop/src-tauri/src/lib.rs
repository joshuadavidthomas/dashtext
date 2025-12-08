mod db;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(db::plugin())
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
