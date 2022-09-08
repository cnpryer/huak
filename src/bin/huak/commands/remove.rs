use std::env;

use super::utils::subcommand;
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::ops;
use huak::{
    errors::{CliError, CliResult},
    project::Project,
};

pub fn arg() -> Command<'static> {
    subcommand("remove")
        .arg(
            Arg::new("dependency")
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .about("Remove a dependency from the project.")
}

pub fn run(args: &ArgMatches) -> CliResult {
    let dependency = match args.get_one::<String>("dependency") {
        Some(d) => d,
        None => {
            return Err(CliError::new(
                anyhow::format_err!("no dependency was provided"),
                2,
            ))
        }
    };
    let cwd = env::current_dir()?;
    let project = Project::new(cwd);

    ops::remove::remove_project_dependency(&project, dependency)?;

    Ok(())
}
