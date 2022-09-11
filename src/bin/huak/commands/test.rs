use super::utils::subcommand;
use clap::Command;
use huak::ops;
use huak::{errors::CliResult, project::Project};
use std::env;

pub fn arg() -> Command<'static> {
    subcommand("test").about("Test Python code.")
}

// TODO: Use pyproject.toml for configuration overrides.
pub fn run() -> CliResult {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = Project::new(cwd);

    ops::test::test_project(&project)?;

    Ok(())
}
