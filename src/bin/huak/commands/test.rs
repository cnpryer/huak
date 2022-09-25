use super::utils::subcommand;
use clap::Command;
use huak::ops;
use huak::{errors::CliResult, project::Project};
use std::env;

/// Get the `test` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("test").about("Test Python code.")
}

/// Run the `test` command.
// TODO: Use pyproject.toml for configuration overrides.
pub fn run() -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    ops::test::test_project(&project)?;

    Ok(())
}
