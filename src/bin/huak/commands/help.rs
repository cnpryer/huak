use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

pub fn arg() -> Command<'static> {
    subcommand("help")
        .about("Display Huak commands and general usage information.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
