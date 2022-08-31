use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

pub fn arg() -> Command<'static> {
    subcommand("activate").about("Activate the project's virtual environment.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
