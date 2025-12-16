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
    registered: AtomicBool,
    stop_flag: Arc<AtomicBool>,
    listener: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
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
            stop_flag: Arc::new(AtomicBool::new(false)),
            listener: Arc::new(Mutex::new(None)),
        })
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

        // Spawn the listener thread
        let listener_handle = std::thread::spawn(move || {
            evdev_listener_loop(devices, app, stop_flag);
        });

        // Store the listener handle
        *self.listener.lock().unwrap() = Some(listener_handle);

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

    // Define the hotkey: Ctrl+Shift+C
    let target_key = Key::KEY_C;
    let modifier_keys: HashSet<Key> = [Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT]
        .iter()
        .copied()
        .collect();

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
                            // We need at least one Ctrl and one Shift
                            let has_ctrl = active_modifiers.contains(&Key::KEY_LEFTCTRL)
                                || active_modifiers.contains(&Key::KEY_RIGHTCTRL);
                            let has_shift = active_modifiers.contains(&Key::KEY_LEFTSHIFT)
                                || active_modifiers.contains(&Key::KEY_RIGHTSHIFT);
                            let modifiers_satisfied = has_ctrl && has_shift;

                            if modifiers_satisfied {
                                match value {
                                    1 if !is_pressed => {
                                        // Key press (not repeat)
                                        is_pressed = true;
                                        tracing::info!("Quick capture hotkey pressed (evdev)");
                                        let _ = app.emit("hotkey:capture:pressed", ());
                                    }
                                    0 if is_pressed => {
                                        // Key release
                                        is_pressed = false;
                                        tracing::debug!("Quick capture hotkey released (evdev)");
                                        let _ = app.emit("hotkey:capture:released", ());
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
