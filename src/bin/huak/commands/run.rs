use super::utils::subcommand;
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::errors::CliResult;
/*
/// Get the `run` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("run")
        .arg(
            Arg::new("command")
                .multiple_values(true)
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .about("Run a command within the project's environment context.")
}
*/
/// Run the `run` command.
pub fn run(args: &ArgMatches) -> CliResult<()> {
    let _ = args.get_many::<String>("command");

    unimplemented!()
}
