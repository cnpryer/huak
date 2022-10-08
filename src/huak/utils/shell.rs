use std::path::Path;

use crate::errors::{HuakError, HuakResult};

/// Gets the name of the current shell.
///
/// Returns an error if it fails to get correct env vars.
pub fn get_shell_name() -> HuakResult<String> {
    let shell_path = get_shell_path()?;
    let shell_name = Path::new(&shell_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_owned())
        .ok_or_else(|| {
            HuakError::InternalError("Shell path is invalid.".to_owned())
        });

    shell_name
}

/// Gets the path of the current shell from env var
///
/// Returns an error if it fails to get correct env vars.
#[cfg(unix)]
pub fn get_shell_path() -> HuakResult<String> {
    Ok(std::env::var("SHELL")?)
}

/// Gets the path of the current shell from env var
///
/// Returns an error if it fails to get correct env vars.
#[cfg(windows)]
pub fn get_shell_path() -> HuakResult<String> {
    Ok(std::env::var("COMSPEC")?)
}

/// Gets the `source` command for the current shell.
///
/// Returns an error if it fails to get correct env vars.
pub fn get_shell_source_command() -> HuakResult<String> {
    let shell_name = get_shell_name()?;

    let command =
        if matches!(shell_name.as_str(), "fish" | "csh" | "tcsh" | "nu") {
            "source"
        } else {
            "."
        };

    Ok(command.to_owned())
}
