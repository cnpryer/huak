use clap::{value_parser, Arg, ArgMatches, Command};
use huak::errors::CliResult;

use crate::utils::subcommand;

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
    let _args = args.get_one::<String>("dependency");

    unimplemented!()
}
