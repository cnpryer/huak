use super::utils::subcommand;
use clap::Command;
use huak::ops;
use huak::{errors::CliResult, project::Project};
use std::env;

pub fn arg() -> Command<'static> {
    subcommand("lint").about("Lint Python code.")
}

// TODO: Use pyproject.toml or .flake8 to override configuration.
pub fn run() -> CliResult {
    // This command runs from the context of the cwd.
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    ops::lint::lint_project(&project)?;

    Ok(())
}
