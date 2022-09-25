use std::env;
use std::process::ExitCode;

use super::utils::subcommand;
use clap::Command;
use huak::errors::{CliError, CliResult};
use huak::ops;
use huak::project::Project;

/// Get the `init` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("init").about("Initialize the existing project.")
}

/// Run the `init` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;

    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) = ops::init::init_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
