use std::{env, process::ExitCode};

use super::utils::subcommand;
use clap::Command;
use huak::{
    errors::{CliError, CliResult},
    ops,
    project::Project,
};

/// Get the `clean` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("clean")
        .about("Remove tarball and wheel from the built project.")
}

/// Run the `clean` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    if let Err(e) = ops::clean::clean_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
