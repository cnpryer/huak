use super::utils::subcommand;
use anyhow::Error;
use clap::Command;
use huak::errors::{CliError, CliResult};
use std::{fs::remove_dir_all, path::Path};

pub fn arg() -> Command<'static> {
    subcommand("clean").about("Remove tarball and wheel from the built project.")
}

pub fn run() -> CliResult {
    if !Path::new("dist").is_dir() {
        Ok(())
    } else {
        match remove_dir_all("dist") {
            Ok(_) => Ok(()),
            Err(e) => Err(CliError {
                exit_code: 2,
                error: Some(Error::new(e)),
            }),
        }
    }
}
