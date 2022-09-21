use std::env;

use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;
use huak::ops;
use huak::project::Project;

/// Get the `install` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("install")
        .about("Install the dependencies of an existing project.")
}

/// Run the `install` command.
pub fn run() -> CliResult {
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    ops::install::install_project_dependencies(&project)?;

    Ok(())
}
