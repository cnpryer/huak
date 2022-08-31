use crate::errors::CliError;
use std::env::{self, consts::OS};

/// Get module filepath from .venv. This function assumes the .venv is in
/// the cwd.
// TODO: Use environment management to determine venv target.
//       This assumes there is a .venv in cwd.
pub fn get_venv_module_path(module: &str) -> Result<String, CliError> {
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();
    let path = cwd.join(".venv").join(get_venv_bin()).join(module);

    if !path.exists() {
        return Err(CliError::new(
            anyhow::format_err!("could not find {}", module),
            2,
        ));
    }

    let path = path.as_os_str().to_str();

    if path.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("failed to create {} path", module),
            2,
        ));
    }

    Ok(path.unwrap().to_string())
}

/// Get the bin or scripts directory based on the OS.
pub fn get_venv_bin() -> String {
    match OS {
        "windows" => "Scripts".to_string(),
        _ => "bin".to_string(),
    }
}
