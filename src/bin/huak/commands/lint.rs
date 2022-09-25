use super::utils::subcommand;
use clap::Command;
use huak::errors::CliError;
use huak::ops;
use huak::{errors::CliResult, project::Project};
use std::env;
use std::process::ExitCode;

/// Get the `lint` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("lint").about("Lint Python code.")
}

/// Run the `lint` command.
pub fn run() -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    if let Err(e) = ops::lint::lint_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
