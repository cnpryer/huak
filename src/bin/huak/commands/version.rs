use std::env;

use clap::Command;
use huak::{
    errors::CliResult,
    ops,
    project::{python::PythonProject, Project},
};

use super::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("version").about("Display the version of the project.")
}

pub fn run() -> CliResult {
    let cwd = env::current_dir()?;
    let project = Project::from(cwd)?;

    let version = ops::version::get_project_version(&project)?;
    let name = &project.config().project_name();

    println!("Version: {name}-{version}");

    Ok(())
}
