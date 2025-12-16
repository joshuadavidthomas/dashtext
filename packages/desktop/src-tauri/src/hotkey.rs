mod global;
#[cfg(target_os = "linux")]
mod evdev;
mod shortcut;

use std::sync::Arc;

/// Trait for cross-platform hotkey management
pub trait HotkeyManager: Send + Sync {
    /// Register the quick-capture hotkey
    fn register(&self) -> Result<(), String>;

    /// Unregister the hotkey
    fn unregister(&self) -> Result<(), String>;
}

/// Create the appropriate HotkeyManager for the current platform
pub fn create_manager(
    app: tauri::AppHandle,
    shortcut: &str,
) -> Result<Arc<dyn HotkeyManager>, String> {
    #[cfg(target_os = "linux")]
    {
        // Try evdev first on Linux
        match evdev::EvdevHotkeyManager::new(app.clone(), shortcut) {
            Ok(manager) => {
                tracing::info!("Using evdev backend for hotkeys");
                return Ok(Arc::new(manager));
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize evdev backend: {}, falling back to Global Shortcut",
                    e
                );
                // Fall back to Tauri Global Shortcut
            }
        }
    }

    // Use Global Shortcut for macOS, Windows, and Linux fallback
    tracing::info!("Using Global Shortcut backend for hotkeys");
    let manager = global::GlobalShortcutManager::new(app, shortcut)?;
    Ok(Arc::new(manager))
}
