use super::utils::subcommand;
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::errors::CliResult;

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

/// Run the `add` subcommand.
pub fn run(args: &ArgMatches) -> CliResult<()> {
    let _ = args.get_one::<String>("dependency");

    unimplemented!()
}
