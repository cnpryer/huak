use std::{path::Path, process};

use crate::errors::CliError;

/// Run a command using process::Command and an array of args. The command will
/// execute inside a `from` dir.
pub(crate) fn run_command(
    cmd: &str,
    args: &[&str],
    from: &Path,
) -> Result<i32, CliError> {
    // Get the child from spawning the process. Child inherets this scopes
    // stdout.
    let mut child = process::Command::new(cmd)
        .args(args)
        .current_dir(from)
        .spawn()?;

    // Get status code for process we're waiting for.
    let status = match child.try_wait() {
        Ok(Some(s)) => s,
        Ok(None) => child.wait()?,
        Err(e) => return Err(CliError::new(anyhow::format_err!(e), 2)),
    };

    Ok(status.code().unwrap_or_default())
}
