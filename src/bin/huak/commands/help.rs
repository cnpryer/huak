use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

/// Get the `help` subcommand.
// Note that this can remain an unimplemented subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("help")
        .about("Display Huak commands and general usage information.")
}

/// Run the `help` command.
pub fn run() -> CliResult<()> {
    unimplemented!()
}
