use std::{env, path::Path, process};

use crate::errors::{HuakError, HuakResult};

/// Run a command using process::Command and an array of args. The command will
/// execute inside a `from` dir. Set the environment variable
/// HUAK_MUTE_COMMAND to True to mute subcommand stdout.
pub(crate) fn run_command(
    cmd: &str,
    args: &[&str],
    from: &Path,
) -> HuakResult<(i32, String)> {
    let (code, msg) = match should_mute() {
        true => run_command_quiet(cmd, args, from)?,
        false => run_command_noisy(cmd, args, from)?,
    };

    if code != 0 {
        // TODO: Capture status codes.
        return Err(HuakError::WrappedCommandError(msg));
    }

    Ok((code, msg))
}

/// Mute command utilities with HUAK_MUTE_COMMAND ("True", "true").
fn should_mute() -> bool {
    let mute = match env::var("HUAK_MUTE_COMMAND") {
        Ok(m) => m,
        Err(_) => "False".to_string(),
    };

    matches!(mute.as_str(), "TRUE" | "True" | "true" | "1")
}

/// Run initialized command with .output() to mute stdout.
fn run_command_quiet(
    cmd: &str,
    args: &[&str],
    from: &Path,
) -> HuakResult<(i32, String)> {
    let output = process::Command::new(cmd)
        .args(args)
        .current_dir(from)
        .output()?;

    let status = output.status;
    let stdout = string_from_buff(&output.stdout)?;
    let stderr = string_from_buff(&output.stderr)?;
    let code = status.code().unwrap_or_default();

    let msg = create_msg(&stdout, &stderr);

    Ok((code, msg))
}

/// Run command and capture entire stdout.
fn run_command_noisy(
    cmd: &str,
    args: &[&str],
    from: &Path,
) -> HuakResult<(i32, String)> {
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
        Err(e) => {
            return Err(HuakError::from(e));
        }
    };

    // TODO: Capture through spawn.
    // Since spawn inherits the user's stdout and stderr, the msg can be empty since
    // it's already displayed.
    let msg = "".to_string();
    let code = status.code().unwrap_or_default();

    Ok((code, msg))
}

fn string_from_buff(buff: &[u8]) -> HuakResult<String> {
    let string = std::str::from_utf8(buff)?.to_string();

    Ok(string)
}

fn create_msg(stdout: &str, stderr: &str) -> String {
    // TODO: Better process context management.
    let mut msg = stdout.to_string();

    if !stderr.is_empty() {
        msg.push('\n');
        msg.push_str(stderr);
    }

    msg
}
