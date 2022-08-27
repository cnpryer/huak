use clap::{arg, value_parser, Arg, ArgAction, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("build")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("Build tarball and wheel for the project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
