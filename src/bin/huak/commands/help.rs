use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("help").about("Display Huak commands and general usage information.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
