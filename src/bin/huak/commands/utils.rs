use std::{path::Path, process::Command};

use huak::errors::CliError;

/// Creates a venv using
pub fn create_venv(python_target: &str, dir_path: &Path, name: &str) -> Result<(), CliError> {
    // While creating the lib path, we're creating the __pypackages__ structure.
    let output = Command::new(python_target)
        .args(&["-m", "venv", name])
        .current_dir(dir_path)
        .output()?;

    if !output.status.success() {
        return Err(CliError::new(
            anyhow::format_err!("failed to create virtual environment"),
            2,
        ));
    }

    Ok(())
}
