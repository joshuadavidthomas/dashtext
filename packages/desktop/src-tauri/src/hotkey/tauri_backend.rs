use super::HotkeyManager;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub struct TauriHotkeyManager {
    app: tauri::AppHandle,
    shortcut: Shortcut,
    registered: AtomicBool,
}

impl TauriHotkeyManager {
    pub fn new(app: tauri::AppHandle, shortcut_str: &str) -> Result<Self, String> {
        let shortcut = Self::parse_shortcut(shortcut_str)?;
        Ok(Self {
            app,
            shortcut,
            registered: AtomicBool::new(false),
        })
    }

    fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
        // Split the shortcut string by '+' to extract modifiers and key
        let parts: Vec<&str> = shortcut_str.split('+').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return Err("Empty shortcut string".to_string());
        }

        let mut modifiers = Modifiers::empty();
        let key_str = parts.last().unwrap();

        // Parse modifiers
        for part in &parts[..parts.len() - 1] {
            match *part {
                "CommandOrControl" => {
                    #[cfg(target_os = "macos")]
                    {
                        modifiers |= Modifiers::SUPER;
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        modifiers |= Modifiers::CONTROL;
                    }
                }
                "Command" | "Cmd" | "Super" => {
                    modifiers |= Modifiers::SUPER;
                }
                "Control" | "Ctrl" => {
                    modifiers |= Modifiers::CONTROL;
                }
                "Shift" => {
                    modifiers |= Modifiers::SHIFT;
                }
                "Alt" | "Option" => {
                    modifiers |= Modifiers::ALT;
                }
                _ => {
                    return Err(format!("Unknown modifier: {}", part));
                }
            }
        }

        // Parse key code
        let code = Self::parse_key_code(key_str)?;

        Ok(Shortcut::new(Some(modifiers), code))
    }

    fn parse_key_code(key: &str) -> Result<Code, String> {
        match key {
            "A" => Ok(Code::KeyA),
            "B" => Ok(Code::KeyB),
            "C" => Ok(Code::KeyC),
            "D" => Ok(Code::KeyD),
            "E" => Ok(Code::KeyE),
            "F" => Ok(Code::KeyF),
            "G" => Ok(Code::KeyG),
            "H" => Ok(Code::KeyH),
            "I" => Ok(Code::KeyI),
            "J" => Ok(Code::KeyJ),
            "K" => Ok(Code::KeyK),
            "L" => Ok(Code::KeyL),
            "M" => Ok(Code::KeyM),
            "N" => Ok(Code::KeyN),
            "O" => Ok(Code::KeyO),
            "P" => Ok(Code::KeyP),
            "Q" => Ok(Code::KeyQ),
            "R" => Ok(Code::KeyR),
            "S" => Ok(Code::KeyS),
            "T" => Ok(Code::KeyT),
            "U" => Ok(Code::KeyU),
            "V" => Ok(Code::KeyV),
            "W" => Ok(Code::KeyW),
            "X" => Ok(Code::KeyX),
            "Y" => Ok(Code::KeyY),
            "Z" => Ok(Code::KeyZ),
            "0" => Ok(Code::Digit0),
            "1" => Ok(Code::Digit1),
            "2" => Ok(Code::Digit2),
            "3" => Ok(Code::Digit3),
            "4" => Ok(Code::Digit4),
            "5" => Ok(Code::Digit5),
            "6" => Ok(Code::Digit6),
            "7" => Ok(Code::Digit7),
            "8" => Ok(Code::Digit8),
            "9" => Ok(Code::Digit9),
            "Space" => Ok(Code::Space),
            "Enter" => Ok(Code::Enter),
            "Escape" => Ok(Code::Escape),
            _ => Err(format!("Unsupported key: {}", key)),
        }
    }
}

impl HotkeyManager for TauriHotkeyManager {
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

        self.app
            .global_shortcut()
            .unregister(self.shortcut)
            .map_err(|e| format!("Failed to unregister shortcut: {}", e))?;

        self.registered.store(false, Ordering::SeqCst);
        tracing::info!("Unregistered global hotkey");
        Ok(())
    }
}
