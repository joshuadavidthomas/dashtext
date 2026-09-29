use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use tauri::Emitter;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_global_shortcut::Shortcut;

use super::HotkeyManager;
use super::shortcut::ShortcutSpec;

pub struct GlobalShortcutManager {
    app: tauri::AppHandle,
    shortcut: Shortcut,
    registered: AtomicBool,
}

impl GlobalShortcutManager {
    pub fn new(app: tauri::AppHandle, shortcut_str: &str) -> Result<Self, String> {
        let spec: ShortcutSpec = shortcut_str.parse()?;
        let shortcut = spec.to_tauri()?;
        Ok(Self {
            app,
            shortcut,
            registered: AtomicBool::new(false),
        })
    }
}

impl HotkeyManager for GlobalShortcutManager {
    fn register(&self) -> Result<(), String> {
        if self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        let app = self.app.clone();

        self.app
            .global_shortcut()
            .on_shortcut(self.shortcut, move |_app, _shortcut, _event| {
                tracing::info!("Quick capture hotkey triggered");
                // Emit event that frontend will listen to
                if let Err(error) = app.emit("hotkey:capture", ()) {
                    tracing::warn!("Failed to emit capture event: {error}");
                }
            })
            .map_err(|e| format!("Failed to register hotkey: {e}"))?;

        self.registered.store(true, Ordering::SeqCst);
        tracing::info!("Registered global hotkey");
        Ok(())
    }

    fn unregister(&self) -> Result<(), String> {
        if !self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.app
            .global_shortcut()
            .unregister(self.shortcut)
            .map_err(|e| format!("Failed to unregister shortcut: {e}"))?;

        self.registered.store(false, Ordering::SeqCst);
        tracing::info!("Unregistered global hotkey");
        Ok(())
    }
}
