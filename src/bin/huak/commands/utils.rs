use clap::{self, App, AppSettings};
use huak::errors::{CliError, CliResult};
use std::{path::Path, process};

/// Create a clap subcommand.
pub fn subcommand(name: &'static str) -> clap::Command<'static> {
    App::new(name)
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
}

/// Creates a venv using python -m venv `name` from a given directory.
pub fn create_venv(python_target: &str, dir: &Path, name: &str) -> CliResult {
    run_command(python_target, &["-m", "venv", name], dir)?;

    Ok(())
}

/// Run a command using std::process::Command
pub fn run_command(command: &str, args: &[&str], dir: &Path) -> CliResult {
    let output = process::Command::new(command)
        .args(args)
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(CliError::new(
            anyhow::format_err!("failed to run command '{}' with {:?}", command, args),
            2,
        ));
    }

    Ok(())
}
