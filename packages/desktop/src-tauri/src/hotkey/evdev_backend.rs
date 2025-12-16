use super::{permissions, HotkeyManager};
use evdev::{Device, InputEventKind, Key};
use std::collections::HashSet;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

pub struct EvdevHotkeyManager {
    app: tauri::AppHandle,
    target_key: Key,
    required_modifiers: HashSet<Key>,
    registered: AtomicBool,
    stop_flag: Arc<AtomicBool>,
    listener: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl EvdevHotkeyManager {
    pub fn new(app: tauri::AppHandle, shortcut_str: &str) -> Result<Self, String> {
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

        // Parse shortcut string
        let (target_key, required_modifiers) = Self::parse_shortcut(shortcut_str)?;

        Ok(Self {
            app,
            target_key,
            required_modifiers,
            registered: AtomicBool::new(false),
            stop_flag: Arc::new(AtomicBool::new(false)),
            listener: Arc::new(Mutex::new(None)),
        })
    }

    fn parse_shortcut(shortcut_str: &str) -> Result<(Key, HashSet<Key>), String> {
        // Split the shortcut string by '+' to extract modifiers and key
        let parts: Vec<&str> = shortcut_str.split('+').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return Err("Empty shortcut string".to_string());
        }

        let mut required_modifiers = HashSet::new();
        let key_str = parts.last().unwrap();

        // Parse modifiers
        for part in &parts[..parts.len() - 1] {
            match *part {
                "CommandOrControl" | "Control" | "Ctrl" => {
                    // For evdev on Linux, we always use Ctrl (not Command)
                    required_modifiers.insert(Key::KEY_LEFTCTRL);
                    required_modifiers.insert(Key::KEY_RIGHTCTRL);
                }
                "Command" | "Cmd" | "Super" => {
                    required_modifiers.insert(Key::KEY_LEFTMETA);
                    required_modifiers.insert(Key::KEY_RIGHTMETA);
                }
                "Shift" => {
                    required_modifiers.insert(Key::KEY_LEFTSHIFT);
                    required_modifiers.insert(Key::KEY_RIGHTSHIFT);
                }
                "Alt" | "Option" => {
                    required_modifiers.insert(Key::KEY_LEFTALT);
                    required_modifiers.insert(Key::KEY_RIGHTALT);
                }
                _ => {
                    return Err(format!("Unknown modifier: {}", part));
                }
            }
        }

        // Parse target key
        let target_key = Self::parse_key(key_str)?;

        Ok((target_key, required_modifiers))
    }

    fn parse_key(key: &str) -> Result<Key, String> {
        match key {
            "A" => Ok(Key::KEY_A),
            "B" => Ok(Key::KEY_B),
            "C" => Ok(Key::KEY_C),
            "D" => Ok(Key::KEY_D),
            "E" => Ok(Key::KEY_E),
            "F" => Ok(Key::KEY_F),
            "G" => Ok(Key::KEY_G),
            "H" => Ok(Key::KEY_H),
            "I" => Ok(Key::KEY_I),
            "J" => Ok(Key::KEY_J),
            "K" => Ok(Key::KEY_K),
            "L" => Ok(Key::KEY_L),
            "M" => Ok(Key::KEY_M),
            "N" => Ok(Key::KEY_N),
            "O" => Ok(Key::KEY_O),
            "P" => Ok(Key::KEY_P),
            "Q" => Ok(Key::KEY_Q),
            "R" => Ok(Key::KEY_R),
            "S" => Ok(Key::KEY_S),
            "T" => Ok(Key::KEY_T),
            "U" => Ok(Key::KEY_U),
            "V" => Ok(Key::KEY_V),
            "W" => Ok(Key::KEY_W),
            "X" => Ok(Key::KEY_X),
            "Y" => Ok(Key::KEY_Y),
            "Z" => Ok(Key::KEY_Z),
            "0" => Ok(Key::KEY_0),
            "1" => Ok(Key::KEY_1),
            "2" => Ok(Key::KEY_2),
            "3" => Ok(Key::KEY_3),
            "4" => Ok(Key::KEY_4),
            "5" => Ok(Key::KEY_5),
            "6" => Ok(Key::KEY_6),
            "7" => Ok(Key::KEY_7),
            "8" => Ok(Key::KEY_8),
            "9" => Ok(Key::KEY_9),
            "Space" => Ok(Key::KEY_SPACE),
            "Enter" => Ok(Key::KEY_ENTER),
            "Escape" => Ok(Key::KEY_ESC),
            _ => Err(format!("Unsupported key: {}", key)),
        }
    }
}

impl HotkeyManager for EvdevHotkeyManager {
    fn register(&self) -> Result<(), String> {
        if self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        let devices = find_keyboard_devices()?;
        if devices.is_empty() {
            return Err("No keyboard devices found".to_string());
        }

        tracing::info!("Found {} keyboard device(s)", devices.len());

        // Reset stop flag
        self.stop_flag.store(false, Ordering::SeqCst);
        
        let stop_flag = self.stop_flag.clone();
        let app = self.app.clone();
        let target_key = self.target_key;
        let required_modifiers = self.required_modifiers.clone();

        // Spawn the listener thread
        let listener_handle = std::thread::spawn(move || {
            evdev_listener_loop(devices, app, stop_flag, target_key, required_modifiers);
        });

        // Store the listener handle
        *self.listener.lock().unwrap() = Some(listener_handle);

        self.registered.store(true, Ordering::SeqCst);
        tracing::info!("Registered global hotkey via evdev");
        Ok(())
    }

    fn is_registered(&self) -> bool {
        self.registered.load(Ordering::SeqCst)
    }

    fn unregister(&self) -> Result<(), String> {
        if !self.registered.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Signal the listener thread to stop
        self.stop_flag.store(true, Ordering::SeqCst);

        // Wait for the thread to finish (with timeout)
        if let Some(handle) = self.listener.lock().unwrap().take() {
            // Give the thread some time to notice the stop flag and exit
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Try to join, but don't block forever
            let _ = handle.join();
        }

        self.registered.store(false, Ordering::SeqCst);
        tracing::info!("Unregistered evdev global hotkey");
        Ok(())
    }
}

/// Find all keyboard input devices by probing /dev/input/event* devices
fn find_keyboard_devices() -> Result<Vec<PathBuf>, String> {
    let mut keyboards = Vec::new();

    let input_dir = std::fs::read_dir("/dev/input")
        .map_err(|e| format!("Failed to read /dev/input: {}", e))?;

    for entry in input_dir {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Only look at event* devices
        let is_event_device = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("event"))
            .unwrap_or(false);

        if !is_event_device {
            continue;
        }

        // Try to open and check if it's a keyboard
        match Device::open(&path) {
            Ok(device) => {
                // Check if device has keyboard capabilities
                let has_keys = device
                    .supported_keys()
                    .map(|keys| {
                        // A keyboard should have at least some letter keys
                        keys.contains(Key::KEY_A)
                            && keys.contains(Key::KEY_Z)
                            && keys.contains(Key::KEY_ENTER)
                    })
                    .unwrap_or(false);

                if has_keys {
                    tracing::debug!(
                        "Found keyboard: {:?} ({:?})",
                        path,
                        device.name().unwrap_or("unknown")
                    );
                    keyboards.push(path);
                }
            }
            Err(e) => {
                // Permission denied is already checked upfront, but log other errors
                if e.kind() != std::io::ErrorKind::PermissionDenied {
                    tracing::trace!("Skipping {:?}: {}", path, e);
                }
            }
        }
    }

    if keyboards.is_empty() {
        return Err("No keyboard devices found with required capabilities".to_string());
    }

    Ok(keyboards)
}

/// Main listener loop running in a blocking thread
fn evdev_listener_loop(
    device_paths: Vec<PathBuf>,
    app: tauri::AppHandle,
    stop_flag: Arc<AtomicBool>,
    target_key: Key,
    modifier_keys: HashSet<Key>,
) {
    // Open all keyboard devices in non-blocking mode
    let mut devices: Vec<Device> = device_paths
        .iter()
        .filter_map(|path| match Device::open(path) {
            Ok(device) => {
                // Set device to non-blocking mode so fetch_events doesn't block
                let fd = device.as_raw_fd();
                unsafe {
                    let flags = libc::fcntl(fd, libc::F_GETFL);
                    if flags != -1 {
                        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                    }
                }
                tracing::debug!("Opened device (non-blocking): {:?}", path);
                Some(device)
            }
            Err(e) => {
                tracing::warn!("Failed to open {:?}: {}", path, e);
                None
            }
        })
        .collect();

    if devices.is_empty() {
        tracing::error!("No keyboard devices could be opened");
        return;
    }

    // Track currently held modifier keys
    let mut active_modifiers: HashSet<Key> = HashSet::new();

    // Track if we're currently "pressed" (to handle repeat events)
    let mut is_pressed = false;

    tracing::info!(
        "Listening for Ctrl+Shift+C (target: {:?}, modifiers: {:?})",
        target_key,
        modifier_keys
    );

    loop {
        // Check for stop signal
        if stop_flag.load(Ordering::SeqCst) {
            tracing::debug!("Hotkey listener stopping");
            return;
        }

        // Poll each device (all set to non-blocking mode)
        for device in &mut devices {
            // fetch_events returns immediately if no events (non-blocking)
            if let Ok(events) = device.fetch_events() {
                for event in events {
                    if let InputEventKind::Key(key) = event.kind() {
                        let value = event.value();

                        // Track modifier state
                        if modifier_keys.contains(&key) {
                            match value {
                                1 => {
                                    active_modifiers.insert(key);
                                }
                                0 => {
                                    active_modifiers.remove(&key);
                                }
                                _ => {}
                            }
                        }

                        // Check target key
                        if key == target_key {
                            // Check if required modifiers are held
                            // At least one of each required modifier type must be held
                            // Group modifiers by type (left/right variants)
                            let mut has_ctrl = false;
                            let mut has_shift = false;
                            let mut has_alt = false;
                            let mut has_meta = false;

                            for modifier in &active_modifiers {
                                match *modifier {
                                    Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => has_ctrl = true,
                                    Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => has_shift = true,
                                    Key::KEY_LEFTALT | Key::KEY_RIGHTALT => has_alt = true,
                                    Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => has_meta = true,
                                    _ => {}
                                }
                            }

                            // Check if all required modifier groups are satisfied
                            let needs_ctrl = modifier_keys.contains(&Key::KEY_LEFTCTRL) 
                                || modifier_keys.contains(&Key::KEY_RIGHTCTRL);
                            let needs_shift = modifier_keys.contains(&Key::KEY_LEFTSHIFT) 
                                || modifier_keys.contains(&Key::KEY_RIGHTSHIFT);
                            let needs_alt = modifier_keys.contains(&Key::KEY_LEFTALT) 
                                || modifier_keys.contains(&Key::KEY_RIGHTALT);
                            let needs_meta = modifier_keys.contains(&Key::KEY_LEFTMETA) 
                                || modifier_keys.contains(&Key::KEY_RIGHTMETA);

                            let modifiers_satisfied = 
                                (!needs_ctrl || has_ctrl) &&
                                (!needs_shift || has_shift) &&
                                (!needs_alt || has_alt) &&
                                (!needs_meta || has_meta);

                            if modifiers_satisfied {
                                match value {
                                    1 if !is_pressed => {
                                        // Key press (not repeat)
                                        is_pressed = true;
                                        tracing::info!("Quick capture hotkey triggered (evdev)");
                                        let _ = app.emit("hotkey:capture", ());
                                    }
                                    0 if is_pressed => {
                                        // Key release
                                        is_pressed = false;
                                        tracing::debug!("Quick capture hotkey released (evdev)");
                                    }
                                    2 => {
                                        // Key repeat - ignore
                                    }
                                    _ => {}
                                }
                            } else if value == 0 && is_pressed {
                                // Released without modifiers held
                                is_pressed = false;
                            }
                        }
                    }
                }
            }
        }

        // Small sleep to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
