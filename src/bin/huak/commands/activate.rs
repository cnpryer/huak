use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("activate").about("Activate the project's virtual environment.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
