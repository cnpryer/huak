use super::utils::subcommand;
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::errors::CliResult;
/*
/// Get the `update` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("update")
        .arg(
            Arg::new("dependency")
                .value_parser(value_parser!(String))
                .default_value("*"),
        )
        .about("Update dependencies added to the project.")
}
*/
/// Run the `update` command.
pub fn run(dependency: String) -> CliResult<()> {
    //let _ = args.get_one::<String>("dependency").unwrap();

    unimplemented!()
}
