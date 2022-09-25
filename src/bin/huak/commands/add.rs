use std::env;
use std::process::ExitCode;

use super::utils::subcommand;
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::errors::{CliResult, CliError, HuakError};
use huak::ops;
use huak::project::Project;

/// Get the `add` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("add")
        .arg(
            Arg::new("dependency")
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .about("Add a Python module to the existing project.")
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let dependency = match args.get_one::<String>("dependency") {
        Some(d) => d,
        None => {
            return Err(CliError::new(
                HuakError::MissingArguments,
                ExitCode::FAILURE,
            ))
        }
    };

    let cwd = env::current_dir()?;
    let project = match Project::from(cwd) {
        Ok(p) => p,
        Err(e) => return Err(CliError::new(e, ExitCode::FAILURE)),
    };

    if let Err(e) = ops::add::add_project_dependency(&project, dependency, false) {
        return Err(CliError::new(e, ExitCode::FAILURE));
    }
    Ok(())
}
