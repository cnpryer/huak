use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

/// Get the `activate` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("activate").about("Activate the project's virtual environment.")
}

/// Run the `activate` command.
pub fn run() -> CliResult<()> {
    unimplemented!()
}
