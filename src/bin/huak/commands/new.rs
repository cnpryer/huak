use std::{fs, path::Path};

use clap::Command;
use huak::{errors::CliResult, pyproject};

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("new").about("Create a project from scratch.")
}

pub fn run() -> CliResult {
    let toml = pyproject::create()?;
    // Creates project directory. TODO: Allow current dir ".".
    let path = Path::new(toml.main().name());

    fs::create_dir_all(path)?;
    fs::write(path.join("pyproject.toml"), toml.to_string())?;

    // Create src subdirectory with standard project namespace.
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("src").join(toml.main().name()))?;

    // Add __init__.py to main project namespace.
    fs::write(
        path.join("src")
            .join(toml.main().name())
            .join("__init__.py"),
        "",
    )?;

    Ok(())
}
