use std::fs;

use clap::Command;
use huak::{
    errors::{CliError, CliResult},
    pyproject::toml::Toml,
};

use super::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("version").about("Display the version of the project.")
}

pub fn run() -> CliResult {
    let string = match fs::read_to_string("pyproject.toml") {
        Ok(s) => s,
        Err(_) => return Err(CliError::new(anyhow::format_err!("failed to read toml"), 2)),
    };

    let toml = match Toml::from(&string) {
        Ok(t) => t,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to build toml"),
                2,
            ))
        }
    };

    println!("{}-{}", toml.tool.huak.name(), toml.tool.huak.version());

    Ok(())
}
