use super::HotkeyManager;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub struct TauriHotkeyManager {
    app: tauri::AppHandle,
    registered: AtomicBool,
}

impl TauriHotkeyManager {
    pub fn new(app: tauri::AppHandle) -> Result<Self, String> {
        Ok(Self {
            app,
            registered: AtomicBool::new(false),
        })
    }

    fn get_shortcut() -> Shortcut {
        #[cfg(target_os = "macos")]
        {
            // Cmd+Shift+C on macOS
            Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyC)
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Ctrl+Shift+C on Windows and Linux
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyC)
        }
    }
}

impl HotkeyManager for TauriHotkeyManager {
    fn register(&self) -> Result<(), String> {
        if self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        let shortcut = Self::get_shortcut();
        let app = self.app.clone();

        self.app
            .global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, _event| {
                tracing::info!("Quick capture hotkey triggered");
                // Emit event that frontend will listen to
                let _ = app.emit("hotkey:capture", ());
            })
            .map_err(|e| format!("Failed to register hotkey: {}", e))?;

        self.registered.store(true, Ordering::SeqCst);
        tracing::info!("Registered global hotkey via Tauri Global Shortcut");
        Ok(())
    }

    fn is_registered(&self) -> bool {
        self.registered.load(Ordering::SeqCst)
    }

    fn unregister(&self) -> Result<(), String> {
        if !self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        let shortcut = Self::get_shortcut();
        self.app
            .global_shortcut()
            .unregister(shortcut)
            .map_err(|e| format!("Failed to unregister shortcut: {}", e))?;

        self.registered.store(false, Ordering::SeqCst);
        tracing::info!("Unregistered global hotkey");
        Ok(())
    }
}
