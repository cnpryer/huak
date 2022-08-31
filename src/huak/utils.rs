use std::env::consts::OS;

/// Get the bin or scripts directory based on the OS.
pub fn get_venv_bin() -> String {
    match OS {
        "windows" => "Scripts".to_string(),
        _ => "bin".to_string(),
    }
}
