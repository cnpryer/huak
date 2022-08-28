use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("build").about("Build tarball and wheel for the project.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
