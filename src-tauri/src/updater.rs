use fs2::FileExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

const UPDATE_MANIFEST_URL: &str =
    "https://github.com/joshuadavidthomas/dashtext/releases/latest/download/latest.json";

/// Minisign public key for verifying update signatures
const UPDATE_PUBLIC_KEY: &str = "RWShIod1Hid9Pszyt9kgjV0pvnWmO2lxd2BLvadhh2JfOM67NmdKDhw/";

/// Platform info from latest.json
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformInfo {
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub signature: Option<String>,
}

/// The latest.json manifest structure
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub pub_date: Option<String>,
    pub platforms: HashMap<String, PlatformInfo>,
}

/// Update info returned to the frontend
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub new_version: String,
    pub release_notes: Option<String>,
    pub download_url: String,
    pub can_auto_update: bool,
}

/// Download progress info
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<u8>,
}

/// Get the current platform key for latest.json
fn get_platform_key() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        "linux-x86_64"
    }
    #[cfg(target_arch = "aarch64")]
    {
        "linux-aarch64"
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        "unknown"
    }
}

/// Get the path to the currently running executable
fn get_current_exe() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))
}

/// Check if we can write to the binary's location (for self-update)
fn can_write_to_binary_location() -> bool {
    let exe = match get_current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Try to write to the directory - if we can write, we can update
    if let Some(parent) = exe.parent() {
        let test_path = parent.join(".dashtext-update-test");
        match File::create(&test_path) {
            Ok(_) => {
                let _ = fs::remove_file(&test_path);
                true
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

/// Get the lock file path for update operations
fn get_lock_file_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not determine local data directory".to_string())?;
    let app_dir = data_dir.join("dashtext");
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    Ok(app_dir.join("update.lock"))
}

/// Acquire the update lock
fn acquire_update_lock() -> Result<File, String> {
    let lock_path = get_lock_file_path()?;
    let lock_file =
        File::create(&lock_path).map_err(|e| format!("Failed to create lock file: {}", e))?;
    lock_file
        .try_lock_exclusive()
        .map_err(|_| "Another update is already in progress".to_string())?;
    Ok(lock_file)
}

/// Fetch the update manifest from the server
async fn fetch_manifest() -> Result<UpdateManifest, String> {
    let response = reqwest::get(UPDATE_MANIFEST_URL)
        .await
        .map_err(|e| format!("Failed to fetch update manifest: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch update manifest: HTTP {}",
            response.status()
        ));
    }

    response
        .json::<UpdateManifest>()
        .await
        .map_err(|e| format!("Failed to parse update manifest: {}", e))
}

/// Compare two semver version strings
fn is_newer_version(current: &str, new: &str) -> bool {
    let current = current.trim_start_matches('v');
    let new = new.trim_start_matches('v');

    match (
        semver::Version::parse(current),
        semver::Version::parse(new),
    ) {
        (Ok(c), Ok(n)) => n > c,
        _ => false, // If parsing fails, assume no update
    }
}

/// Download a file to a path with progress reporting
async fn download_file(app: &AppHandle, url: &str, dest: &PathBuf) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download update: HTTP {}",
            response.status()
        ));
    }

    let total_size = response.content_length();
    let mut downloaded: u64 = 0;

    let mut file =
        File::create(dest).map_err(|e| format!("Failed to create download file: {}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded += chunk.len() as u64;

        let _ = app.emit(
            "update-progress",
            DownloadProgress {
                downloaded,
                total: total_size,
                percent: total_size.map(|t| ((downloaded * 100) / t) as u8),
            },
        );
    }

    Ok(())
}

/// Verify that a binary executes successfully with --version
fn verify_binary(path: &PathBuf) -> Result<(), String> {
    let output = std::process::Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute new binary: {}", e))?;

    if !output.status.success() {
        return Err("New binary exited with error".to_string());
    }

    Ok(())
}

/// Verify minisign signature of downloaded file
fn verify_signature(file_path: &PathBuf, signature: &str) -> Result<(), String> {
    use minisign_verify::{PublicKey, Signature};

    // Parse the public key
    let pk = PublicKey::from_base64(UPDATE_PUBLIC_KEY)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    // Parse the signature
    let sig = Signature::decode(signature)
        .map_err(|e| format!("Invalid signature format: {}", e))?;

    // Read the file
    let data = fs::read(file_path)
        .map_err(|e| format!("Failed to read file for signature verification: {}", e))?;

    // Verify
    pk.verify(&data, &sig, false)
        .map_err(|_| "Signature verification failed - update may be tampered".to_string())
}

/// Verify the SHA256 checksum of a file
fn verify_checksum(file_path: &PathBuf, expected_sha256: &str) -> Result<(), String> {
    let mut file =
        File::open(file_path).map_err(|e| format!("Failed to open file for verification: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let actual_hex = hex::encode(result);

    if actual_hex.to_lowercase() != expected_sha256.to_lowercase() {
        return Err(format!(
            "Checksum verification failed. Expected: {}, Got: {}",
            expected_sha256, actual_hex
        ));
    }

    Ok(())
}

/// Extract a tar.gz archive and return the path to the binary
fn extract_tarball(tarball_path: &PathBuf, dest_dir: &PathBuf) -> Result<PathBuf, String> {
    let file =
        File::open(tarball_path).map_err(|e| format!("Failed to open tarball: {}", e))?;

    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(dest_dir)
        .map_err(|e| format!("Failed to extract tarball: {}", e))?;

    // Find the dashtext binary in the extracted contents
    let binary_path = dest_dir.join("dashtext");
    if binary_path.exists() {
        return Ok(binary_path);
    }

    // Maybe it's in a subdirectory
    for entry in fs::read_dir(dest_dir).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            let nested_binary = path.join("dashtext");
            if nested_binary.exists() {
                return Ok(nested_binary);
            }
        }
    }

    Err("Could not find dashtext binary in archive".to_string())
}

// ============ TAURI COMMANDS ============

/// Check for available updates
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let current_version = app.package_info().version.to_string();

    let manifest = fetch_manifest().await?;

    if !is_newer_version(&current_version, &manifest.version) {
        return Ok(None);
    }

    let platform_key = get_platform_key();
    let platform_info = manifest
        .platforms
        .get(platform_key)
        .ok_or_else(|| format!("No update available for platform: {}", platform_key))?;

    Ok(Some(UpdateInfo {
        current_version,
        new_version: manifest.version,
        release_notes: manifest.notes,
        download_url: platform_info.url.clone(),
        can_auto_update: can_write_to_binary_location(),
    }))
}

/// Check if self-update is possible (binary in writable location)
#[tauri::command]
pub fn can_auto_update() -> bool {
    can_write_to_binary_location()
}

/// Download and install the update
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    // Acquire lock to prevent concurrent updates
    let _lock = acquire_update_lock()?;

    let current_version = app.package_info().version.to_string();
    let manifest = fetch_manifest().await?;

    if !is_newer_version(&current_version, &manifest.version) {
        return Err("Already up to date".to_string());
    }

    let platform_key = get_platform_key();
    let platform_info = manifest
        .platforms
        .get(platform_key)
        .ok_or_else(|| format!("No update available for platform: {}", platform_key))?;

    if !can_write_to_binary_location() {
        return Err(
            "Cannot self-update: binary is in a system location. Please update manually."
                .to_string(),
        );
    }

    let current_exe = get_current_exe()?;

    // Create temp directory for download
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let tarball_path = temp_dir.path().join("update.tar.gz");

    // Download the update
    download_file(&app, &platform_info.url, &tarball_path).await?;

    // Verify signature (authenticity)
    let signature = platform_info
        .signature
        .as_ref()
        .ok_or_else(|| "Update missing signature - refusing to install unsigned update".to_string())?;
    verify_signature(&tarball_path, signature)?;

    // Verify checksum (integrity)
    verify_checksum(&tarball_path, &platform_info.sha256)?;

    // Extract the tarball
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    let new_binary = extract_tarball(&tarball_path, &extract_dir)?;

    // Make the new binary executable
    let mut perms = fs::metadata(&new_binary)
        .map_err(|e| format!("Failed to get binary metadata: {}", e))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&new_binary, perms)
        .map_err(|e| format!("Failed to set binary permissions: {}", e))?;

    // Prepare paths for atomic swap
    let backup_path = current_exe.with_extension("old");
    let temp_new_path = current_exe.with_extension("new");

    // Copy new binary to same directory (for atomic rename)
    fs::copy(&new_binary, &temp_new_path)
        .map_err(|e| format!("Failed to copy new binary: {}", e))?;

    // Make the copied binary executable
    let mut perms = fs::metadata(&temp_new_path)
        .map_err(|e| format!("Failed to get temp binary metadata: {}", e))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_new_path, perms)
        .map_err(|e| format!("Failed to set temp binary permissions: {}", e))?;

    // Remove old backup if it exists
    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path);
    }

    // Atomic swap: current -> backup, new -> current
    fs::rename(&current_exe, &backup_path)
        .map_err(|e| format!("Failed to backup current binary: {}", e))?;

    if let Err(e) = fs::rename(&temp_new_path, &current_exe) {
        // Rollback: restore from backup
        let _ = fs::rename(&backup_path, &current_exe);
        return Err(format!("Failed to install new binary: {}", e));
    }

    // Verify the new binary works before cleaning up backup
    if let Err(e) = verify_binary(&current_exe) {
        // Rollback: restore the backup
        let _ = fs::rename(&current_exe, &temp_new_path); // Move bad binary out
        let _ = fs::rename(&backup_path, &current_exe);   // Restore good binary
        let _ = fs::remove_file(&temp_new_path);          // Clean up bad binary
        return Err(format!("Update verification failed, rolled back: {}", e));
    }

    // Success - clean up backup
    let _ = fs::remove_file(&backup_path);

    Ok(())
}

/// Restart the application (exec into new binary)
#[tauri::command]
pub fn restart_app() -> Result<(), String> {
    let current_exe = get_current_exe()?;

    // On Unix, we can exec into the new binary
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&current_exe).exec();
        // exec() only returns if there's an error
        return Err(format!("Failed to restart: {}", err));
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, just spawn and exit
        std::process::Command::new(&current_exe)
            .spawn()
            .map_err(|e| format!("Failed to restart: {}", e))?;
        std::process::exit(0);
    }
}

/// Get the current version
#[tauri::command]
pub fn get_current_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}
