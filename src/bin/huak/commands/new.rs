use clap::{arg, value_parser, Arg, ArgAction, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("new")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("Create a project from scratch.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
