use super::utils::subcommand;
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::errors::CliResult;

pub fn arg() -> Command<'static> {
    subcommand("update")
        .arg(
            Arg::new("dependency")
                .value_parser(value_parser!(String))
                .default_value("*"),
        )
        .about("Update dependencies added to the project.")
}

pub fn run(args: &ArgMatches) -> CliResult {
    let _ = args.get_one::<String>("dependency").unwrap();

    unimplemented!()
}
