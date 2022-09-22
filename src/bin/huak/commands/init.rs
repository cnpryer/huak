use std::env;

use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;
use huak::ops;
use huak::project::Project;

/// Get the `init` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("init").about("Initialize the existing project.")
}

/// Run the `init` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;

    let project = Project::from(cwd)?;

    ops::init::init_project(&project)?;

    Ok(())
}
