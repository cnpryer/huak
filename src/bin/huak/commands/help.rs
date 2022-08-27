use clap::{arg, value_parser, Arg, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("help")
        .value_parser(value_parser!(bool))
        .action(clap::ArgAction::SetTrue)
        .help("Display Huak commands and general usage information.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
