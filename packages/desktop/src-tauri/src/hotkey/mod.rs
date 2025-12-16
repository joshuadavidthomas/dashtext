mod shortcut;
mod tauri_backend;

#[cfg(target_os = "linux")]
mod evdev_backend;

#[cfg(target_os = "linux")]
mod permissions;

use std::sync::Arc;

/// Trait for cross-platform hotkey management
pub trait HotkeyManager: Send + Sync {
    /// Register the quick-capture hotkey
    fn register(&self) -> Result<(), String>;
    
    /// Check if the hotkey is registered
    fn is_registered(&self) -> bool;
    
    /// Unregister the hotkey
    fn unregister(&self) -> Result<(), String>;
}

/// Create the appropriate HotkeyManager for the current platform
pub fn create_manager(app: tauri::AppHandle, shortcut: &str) -> Result<Arc<dyn HotkeyManager>, String> {
    #[cfg(target_os = "linux")]
    {
        // Try evdev first on Linux
        match evdev_backend::EvdevHotkeyManager::new(app.clone(), shortcut) {
            Ok(manager) => {
                tracing::info!("Using evdev backend for hotkeys");
                return Ok(Arc::new(manager));
            }
            Err(e) => {
                tracing::warn!("Failed to initialize evdev backend: {}, falling back to Tauri Global Shortcut", e);
                // Fall back to Tauri Global Shortcut
            }
        }
    }
    
    // Use Tauri Global Shortcut for macOS, Windows, and Linux fallback
    tracing::info!("Using Tauri Global Shortcut backend for hotkeys");
    let manager = tauri_backend::TauriHotkeyManager::new(app, shortcut)?;
    Ok(Arc::new(manager))
}
