use std::str::FromStr;

/// Platform-agnostic shortcut specification parsed from a string like "CommandOrControl+Shift+C"
#[derive(Debug, Clone)]
pub struct ShortcutSpec {
    pub key: String,
    pub ctrl_or_cmd: bool,
    pub meta: bool,
    pub shift: bool,
    pub alt: bool,
}

impl FromStr for ShortcutSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();

        if parts.is_empty() {
            return Err("Empty shortcut string".into());
        }

        let mut spec = Self {
            key: parts.last().unwrap().to_string(),
            ctrl_or_cmd: false,
            meta: false,
            shift: false,
            alt: false,
        };

        // Parse modifiers (all parts except the last)
        for part in &parts[..parts.len() - 1] {
            match *part {
                "CommandOrControl" | "CmdOrCtrl" => spec.ctrl_or_cmd = true,
                "Control" | "Ctrl" => spec.ctrl_or_cmd = true,
                "Command" | "Cmd" | "Super" => spec.meta = true,
                "Shift" => spec.shift = true,
                "Alt" | "Option" => spec.alt = true,
                _ => return Err(format!("Unknown modifier: {}", part)),
            }
        }

        Ok(spec)
    }
}

#[cfg(target_os = "linux")]
impl ShortcutSpec {
    /// Convert to evdev Key and modifiers
    pub fn to_evdev(
        &self,
    ) -> Result<(evdev::Key, std::collections::HashSet<evdev::Key>), String> {
        use evdev::Key;

        // Normalize key name to evdev format (KEY_*)
        let key_name = match self.key.as_str() {
            "Escape" => "KEY_ESC".to_string(),
            "Enter" => "KEY_ENTER".to_string(),
            "Space" => "KEY_SPACE".to_string(),
            k => format!("KEY_{}", k.to_uppercase()),
        };

        let key = Key::from_str(&key_name)
            .map_err(|_| format!("Unsupported key: {}", self.key))?;

        // Build modifier set
        let mut mods = std::collections::HashSet::new();

        // On Linux, CommandOrControl always means Ctrl (not Super/Meta)
        if self.ctrl_or_cmd {
            mods.insert(Key::KEY_LEFTCTRL);
            mods.insert(Key::KEY_RIGHTCTRL);
        }

        if self.meta {
            mods.insert(Key::KEY_LEFTMETA);
            mods.insert(Key::KEY_RIGHTMETA);
        }

        if self.shift {
            mods.insert(Key::KEY_LEFTSHIFT);
            mods.insert(Key::KEY_RIGHTSHIFT);
        }

        if self.alt {
            mods.insert(Key::KEY_LEFTALT);
            mods.insert(Key::KEY_RIGHTALT);
        }

        Ok((key, mods))
    }
}

impl ShortcutSpec {
    /// Convert to Tauri's Shortcut type
    pub fn to_tauri(&self) -> Result<tauri_plugin_global_shortcut::Shortcut, String> {
        use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

        // Normalize key name to Code format (KeyA, Digit1, Space, etc.)
        let code_name = match self.key.as_str() {
            s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
                format!("Key{}", s.to_uppercase())
            }
            s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
                format!("Digit{}", s)
            }
            s => s.to_string(),
        };

        let code = Code::from_str(&code_name)
            .map_err(|e| format!("Unsupported key: {} ({:?})", self.key, e))?;

        // Build modifiers
        let mut mods = Modifiers::empty();

        if self.ctrl_or_cmd {
            #[cfg(target_os = "macos")]
            {
                mods |= Modifiers::SUPER;
            }
            #[cfg(not(target_os = "macos"))]
            {
                mods |= Modifiers::CONTROL;
            }
        }

        if self.meta {
            mods |= Modifiers::SUPER;
        }

        if self.shift {
            mods |= Modifiers::SHIFT;
        }

        if self.alt {
            mods |= Modifiers::ALT;
        }

        Ok(Shortcut::new(Some(mods), code))
    }
}
