use std::{path::Path, process};

use crate::errors::{CliError, CliResult};

/// Run a command using process::Command and an array of args. The command will
/// execute inside a `from` dir.
pub(crate) fn run_command(cmd: &str, args: &[&str], from: &Path) -> CliResult {
    let output = process::Command::new(cmd)
        .args(args)
        .current_dir(from)
        .output()?;

    if !output.status.success() {
        return Err(CliError::new(
            anyhow::format_err!(
                "failed to run command '{}' with {:?}",
                cmd,
                args
            ),
            2,
        ));
    }

    Ok(())
}
