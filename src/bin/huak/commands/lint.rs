use super::utils::subcommand;
use clap::Command;
use huak::ops;
use huak::{errors::CliResult, project::Project};
use std::env;

/// Get the `lint` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("lint").about("Lint Python code.")
}

/// Run the `lint` command.
pub fn run() -> CliResult {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    ops::lint::lint_project(&project)?;

    Ok(())
}
