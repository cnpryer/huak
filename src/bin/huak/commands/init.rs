use std::env;

use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;
use huak::ops;
use huak::project::Project;

pub fn arg() -> Command<'static> {
    subcommand("init").about("Initialize the existing project.")
}

pub fn run() -> CliResult {
    let cwd = env::current_dir()?;

    let project = Project::new(cwd);

    ops::init::create_project_toml(&project)?;
    Ok(())
}
