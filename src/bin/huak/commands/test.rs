use super::utils::subcommand;
use clap::Command;
use huak::errors::CliError;
use huak::ops;
use huak::{errors::CliResult, project::Project};
use std::env;
use std::process::ExitCode;
/*
/// Get the `test` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("test").about("Test Python code.")
}
*/
/// Run the `test` command.
// TODO: Use pyproject.toml for configuration overrides.
pub fn run() -> CliResult<()> {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) = ops::test::test_project(&project) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    };

    Ok(())
}
