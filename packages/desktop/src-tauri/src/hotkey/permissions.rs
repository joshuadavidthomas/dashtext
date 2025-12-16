use std::process::Command;

/// Check if the current user is in the 'input' group
pub fn check_input_group() -> Result<bool, String> {
    let output = Command::new("groups")
        .output()
        .map_err(|e| format!("Failed to execute 'groups' command: {}", e))?;

    if !output.status.success() {
        return Err("'groups' command failed".to_string());
    }

    let groups = String::from_utf8_lossy(&output.stdout);
    Ok(groups.split_whitespace().any(|g| g == "input"))
}

/// Get the current username
pub fn get_username() -> Result<String, String> {
    let output = Command::new("whoami")
        .output()
        .map_err(|e| format!("Failed to execute 'whoami' command: {}", e))?;

    if !output.status.success() {
        return Err("'whoami' command failed".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get a helpful error message for missing input group permissions
pub fn get_permission_error_message() -> String {
    let username = get_username().unwrap_or_else(|_| "your_username".to_string());
    
    format!(
        "evdev requires input group permissions. Please run:\n\
         sudo usermod -aG input {}\n\
         Then log out and log back in for changes to take effect.",
        username
    )
}
