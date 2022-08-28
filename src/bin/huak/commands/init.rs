use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("init").about("Initialize the existing project.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
