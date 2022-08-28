use clap::Command;
use huak::errors::CliResult;
use anyhow::Error;
use std::fs::remove_dir_all;

use huak::errors::{CliError};

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("clean").about("Remove tarball and wheel from the built project.")
}

pub fn run() -> CliResult {
    match remove_dir_all("dist") {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError {
            exit_code: 2,
            error: Some(Error::new(e)),
        }),
    }
}
