use super::{permissions, HotkeyManager};
use evdev_shortcut::{Key, Modifier, Shortcut, ShortcutListener};
use futures::stream::StreamExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

pub struct EvdevHotkeyManager {
    app: tauri::AppHandle,
    registered: AtomicBool,
    listener: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl EvdevHotkeyManager {
    pub fn new(app: tauri::AppHandle) -> Result<Self, String> {
        // Check for input group permissions
        match permissions::check_input_group() {
            Ok(true) => {
                tracing::info!("User has input group permissions");
            }
            Ok(false) => {
                let msg = permissions::get_permission_error_message();
                tracing::warn!("{}", msg);
                return Err(msg);
            }
            Err(e) => {
                tracing::warn!("Failed to check input group: {}", e);
                return Err(format!("Failed to check permissions: {}", e));
            }
        }

        Ok(Self {
            app,
            registered: AtomicBool::new(false),
            listener: Arc::new(Mutex::new(None)),
        })
    }

    fn get_keyboard_devices() -> Result<Vec<PathBuf>, String> {
        let pattern = "/dev/input/by-id/*-kbd";
        glob::glob(pattern)
            .map_err(|e| format!("Failed to glob keyboard devices: {}", e))?
            .collect::<Result<Vec<PathBuf>, _>>()
            .map_err(|e| format!("Failed to collect keyboard devices: {}", e))
    }
}

impl HotkeyManager for EvdevHotkeyManager {
    fn register(&self) -> Result<(), String> {
        if self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        let devices = Self::get_keyboard_devices()?;
        if devices.is_empty() {
            return Err("No keyboard devices found".to_string());
        }

        tracing::info!("Found {} keyboard device(s)", devices.len());

        let shortcut_listener = ShortcutListener::new();
        
        // Ctrl+Shift+C
        let shortcut = Shortcut::new(&[Modifier::Ctrl, Modifier::Shift], Key::KeyC);
        shortcut_listener.add(shortcut);

        let stream = shortcut_listener
            .listen(&devices)
            .map_err(|e| format!("Failed to start listening for shortcuts: {}", e))?;

        let app = self.app.clone();
        let listener_handle = tokio::spawn(async move {
            tokio::pin!(stream);
            
            while let Some(event) = stream.next().await {
                tracing::info!("Shortcut event: {} {}", event.shortcut, event.state);
                
                // Only trigger on press, not release
                if matches!(event.state, evdev_shortcut::ShortcutState::Pressed) {
                    tracing::info!("Quick capture hotkey triggered (evdev)");
                    let _ = app.emit("hotkey:capture", ());
                }
            }
            
            tracing::warn!("evdev shortcut listener stopped");
        });

        // Store the listener handle
        let listener_clone = self.listener.clone();
        tokio::spawn(async move {
            *listener_clone.lock().await = Some(listener_handle);
        });

        self.registered.store(true, Ordering::SeqCst);
        tracing::info!("Registered global hotkey via evdev (Ctrl+Shift+C)");
        Ok(())
    }

    fn is_registered(&self) -> bool {
        self.registered.load(Ordering::SeqCst)
    }

    fn unregister(&self) -> Result<(), String> {
        if !self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Abort the listener task
        let listener = self.listener.clone();
        tokio::spawn(async move {
            if let Some(handle) = listener.lock().await.take() {
                handle.abort();
            }
        });

        self.registered.store(false, Ordering::SeqCst);
        tracing::info!("Unregistered evdev global hotkey");
        Ok(())
    }
}
