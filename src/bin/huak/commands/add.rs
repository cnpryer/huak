use clap::{arg, value_parser, Arg, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("add")
        .value_parser(value_parser!(String))
        .help("Add a Python module to the existing project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
