use std::env;

use super::utils::subcommand;
use clap::Command;
use huak::{errors::CliResult, ops, project::Project};

pub fn arg() -> Command<'static> {
    subcommand("clean")
        .about("Remove tarball and wheel from the built project.")
}

pub fn run() -> CliResult {
    let cwd = env::current_dir()?;
    let project = Project::new(cwd);

    ops::clean::clean_project(&project)?;

    Ok(())
}
