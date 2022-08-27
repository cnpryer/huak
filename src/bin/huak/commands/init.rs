use clap::{arg, value_parser, Arg, ArgAction, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("init")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("Initialize the existing project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
