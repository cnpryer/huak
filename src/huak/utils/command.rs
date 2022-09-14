use std::{env, path::Path, process};

use crate::errors::CliError;

/// Run a command using process::Command and an array of args. The command will
/// execute inside a `from` dir. Set the environment variable
/// HUAK_MUTE_SUBCOMMAND to True to mute subcommand stdout.
pub(crate) fn run_command(
    cmd: &str,
    args: &[&str],
    from: &Path,
) -> Result<i32, CliError> {
    match should_mute() {
        true => run_command_with_output(cmd, args, from),
        false => run_command_with_spawn(cmd, args, from),
    }
}

/// Mute command utilities with HUAK_MUTE_SUBCOMMAND ("True", "true").
fn should_mute() -> bool {
    let _mute = match env::var("HUAK_MUTE_SUBCOMMAND") {
        Ok(m) => m,
        Err(_) => "False".to_string(),
    };

    matches!(_mute.as_str(), "TRUE" | "True" | "true" | "1")
}

/// Run initilized command with .output() to mute stdout.
fn run_command_with_output(
    cmd: &str,
    args: &[&str],
    from: &Path,
) -> Result<i32, CliError> {
    let output = process::Command::new(cmd)
        .args(args)
        .current_dir(from)
        .output()?;

    let status = output.status;

    Ok(status.code().unwrap_or_default())
}

/// Run a command using process::Command and an array of args. The command will
/// execute inside a `from` dir.
pub(crate) fn run_command_with_spawn(
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
