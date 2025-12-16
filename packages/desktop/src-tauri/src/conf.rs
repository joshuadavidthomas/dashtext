use crate::hotkey::{create_manager, HotkeyManager};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

/// Default capture shortcut
pub const DEFAULT_CAPTURE_SHORTCUT: &str = "CommandOrControl+Shift+C";

/// Application settings state
/// Manages active hotkey registration (database persistence handled by frontend)
pub struct SettingsState {
    hotkey_manager: Mutex<Option<Arc<dyn HotkeyManager>>>,
    app: AppHandle,
}

impl SettingsState {
    /// Create new settings state
    pub fn new(app: AppHandle) -> Self {
        Self {
            hotkey_manager: Mutex::new(None),
            app,
        }
    }

    /// Register capture shortcut hotkey
    /// This is called by the frontend after loading settings from the database
    pub async fn register_capture_shortcut(&self, shortcut: &str) -> Result<(), String> {
        // Unregister existing hotkey if any
        {
            let mut mgr = self.hotkey_manager.lock().await;
            if let Some(ref m) = *mgr {
                m.unregister()?;
            }
            *mgr = None;
        }

        // Register new hotkey
        let mgr = create_manager(self.app.clone(), shortcut)?;
        mgr.register()?;
        *self.hotkey_manager.lock().await = Some(mgr);
        
        tracing::info!("Registered capture shortcut: {}", shortcut);
        Ok(())
    }

    /// Unregister capture shortcut
    pub async fn unregister_capture_shortcut(&self) -> Result<(), String> {
        let mut mgr = self.hotkey_manager.lock().await;
        if let Some(ref m) = *mgr {
            m.unregister()?;
            tracing::info!("Unregistered capture shortcut");
        }
        *mgr = None;
        Ok(())
    }
}
