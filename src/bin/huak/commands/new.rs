use std::{fs, path::Path};

use clap::Command;
use huak::errors::{CliError, CliResult};

use crate::pyproject;
use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("new").about("Create a project from scratch.")
}

pub fn run() -> CliResult {
    let toml = pyproject::toml::create()?;
    // Creates project directory. TODO: Allow current dir ".".
    let path = Path::new(toml.tool().huak().name());

    fs::create_dir_all(path)?;
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to serialize toml"),
                2,
            ))
        }
    };
    fs::write(path.join("pyproject.toml"), string)?;

    // Create src subdirectory with standard project namespace.
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("src").join(toml.tool().huak().name()))?;

    // Add __init__.py to main project namespace.
    fs::write(
        path.join("src")
            .join(toml.tool().huak().name())
            .join("__init__.py"),
        "",
    )?;

    Ok(())
}
