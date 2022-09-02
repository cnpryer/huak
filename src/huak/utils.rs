use crate::errors::CliError;
use std::{
    env::{self, consts::OS},
    path::{Path, PathBuf},
};

/// Get module filepath from .venv. This function assumes the .venv is in
/// the cwd.
// TODO: Use environment management to determine venv target.
//       This assumes there is a .venv in cwd.
pub fn get_venv_module_path(module: &str) -> Result<PathBuf, CliError> {
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();
    let path = cwd.join(".venv").join(get_venv_bin()).join(module);

    if !path.exists() {
        return Err(CliError::new(
            anyhow::format_err!("could not find {}", module),
            2,
        ));
    }

    Ok(path)
}

/// Get the bin or scripts directory based on the OS.
pub fn get_venv_bin() -> String {
    match OS {
        "windows" => "Scripts".to_string(),
        _ => "bin".to_string(),
    }
}

/// Return the filename from a `Path`.
pub fn get_filename_from_path(path: &Path) -> Result<String, CliError> {
    // Attempt to convert OsStr to str.
    let name = match path.file_name() {
        Some(f) => f.to_str(),
        _ => {
            return Err(CliError::new(
                anyhow::format_err!("failed to read name from path"),
                2,
            ))
        }
    };

    // If a str was failed to be parsed error.
    if name.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("failed to read name from path"),
            2,
        ));
    }

    Ok(name.unwrap().to_string())
}
