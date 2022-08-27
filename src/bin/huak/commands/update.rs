use clap::{arg, value_parser, Arg, ArgMatches};
use huak::errors::CliResult;

pub fn arg() -> Arg<'static> {
    arg!("update")
        .value_parser(value_parser!(String))
        .default_value("*")
        .help("Update dependencies added to the project.")
}

pub fn run(_args: &ArgMatches) -> CliResult {
    unimplemented!()
}
