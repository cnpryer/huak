use super::utils::subcommand;
use clap::Command;
use huak::errors::CliResult;

pub fn arg() -> Command<'static> {
    subcommand("build").about("Build tarball and wheel for the project.")
}

pub fn run() -> CliResult {
    unimplemented!()
}
