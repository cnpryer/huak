use std::env;
use std::fs;

use super::utils::subcommand;
use clap::{arg, value_parser, ArgMatches, Command};
use huak::errors::{CliError, CliResult, HuakError};
use huak::ops;
use huak::project::Project;

/// Get the `new` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("new")
        .about("Create a project from scratch.")
        .arg(arg!([PATH]).id("path").value_parser(value_parser!(String)))
}

/// Run the `new` command.
// TODO: Ops should hanlde the path creation step in addition to the project creation.
pub fn run(args: &ArgMatches) -> CliResult<()> {
    // This command runs from the current working directory
    // Each command's behavior is triggered from the context of the cwd.
    let cwd = env::current_dir()?;

    // If a user passes a path
    let path = match args.get_one::<String>("path") {
        Some(p) => cwd.join(p),
        _ => cwd.clone(),
    };

    // Make sure there isn't already a path we would override.
    if path.exists() && path != cwd {
        return Err(CliError::new(HuakError::DirectoryExists));
    }

    // If the current directory is used it must be empty. User should use init.
    if path == cwd && path.read_dir()?.count() > 0 {
        return Err(CliError::new(HuakError::DirectoryExists));
    }

    // Create project directory.
    if path != cwd {
        fs::create_dir_all(&path)?;
    }

    let project = Project::from(path)?;

    ops::new::create_project(&project)?;

    Ok(())
}
