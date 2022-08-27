use clap::{arg, value_parser, Arg, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("remove")
        .value_parser(value_parser!(String))
        .help("Remove a dependency from the project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
