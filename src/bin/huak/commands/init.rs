use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

pub fn arg() -> Command<'static> {
    subcommand("init").about("Initialize the existing project.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
