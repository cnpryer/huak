use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("clean").about("Remove tarball and wheel from the built project.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
