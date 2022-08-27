use clap::{arg, value_parser, Arg, ArgAction, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("clean")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("Remove tarball and wheel from the built project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
