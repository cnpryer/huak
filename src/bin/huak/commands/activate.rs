use clap::{arg, value_parser, Arg, ArgAction, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("activate")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("Activate the project's virtual environment.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
