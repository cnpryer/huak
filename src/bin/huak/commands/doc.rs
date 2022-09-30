use super::utils::subcommand;
use clap::{arg, ArgAction, ArgMatches, Command};
use huak::errors::CliResult;
/*
/// Get the `doc` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("doc")
        .arg(
            arg!(--check)
                .id("check")
                .takes_value(false)
                .action(ArgAction::SetTrue)
                .help("Check if Python code is formatted."),
        )
        .about("Builds and uploads current project to a registry.")
}
*/
/// Run the `doc` command.
pub fn run(is_check: bool) -> CliResult<()> {
    // TODO: Use is_check.
    // let _ = args.get_one::<bool>("check").unwrap();

    unimplemented!()
}
