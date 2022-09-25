use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

/// Get the `publish` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("publish")
        .about("Builds and uploads current project to a registry.")
}

/// Run the `publish` command.
pub fn run() -> CliResult<()> {
    unimplemented!()
}
