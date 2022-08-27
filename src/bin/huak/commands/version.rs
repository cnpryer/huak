use clap::{arg, value_parser, Arg, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("version")
        .value_parser(value_parser!(bool))
        .action(clap::ArgAction::SetTrue)
        .help("Display the version of the project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
