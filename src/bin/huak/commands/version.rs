use std::{env, process::ExitCode};

use clap::Command;
use huak::{
    errors::{CliError, CliResult},
    ops,
    project::Project,
};

use super::utils::subcommand;

/// Get the `version` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("version").about("Display the version of the project.")
}

/// Run the `version` command.
pub fn run() -> CliResult<()> {
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    let version = match ops::version::get_project_version(&project) {
        Ok(v) => v,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };
    let name = &project.config().project_name();

    println!("Version: {name}-{version}");

    Ok(())
}
