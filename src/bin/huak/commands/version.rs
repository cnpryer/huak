use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("version").about("Display the version of the project.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
