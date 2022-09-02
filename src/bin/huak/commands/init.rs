use std::{env, fs};

use crate::pyproject::toml::{
    create_authors, create_dependencies, create_description, create_version,
};

use super::utils::subcommand;
use clap::Command;
use huak::{
    errors::{CliError, CliResult},
    pyproject::toml::Toml,
};

pub fn arg() -> Command<'static> {
    subcommand("init").about("Initialize the existing project.")
}

pub fn run() -> CliResult {
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();
    let toml_path = cwd.join("pyproject.toml");

    // Check to see if a pyproject.toml already exists in cwd.
    if toml_path.exists() {
        return Err(CliError::new(
            anyhow::format_err!("a pyproject.toml already exists"),
            2,
        ));
    }

    let name = toml_path.parent();

    if name.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("could not initialize the project name"),
            2,
        ));
    }

    let name = name.unwrap().to_str();

    if name.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("could not initialize the project name"),
            2,
        ));
    }

    let mut toml = Toml::new();

    toml.tool.huak.set_name(name.unwrap().to_string());
    toml.tool.huak.set_version(create_version()?);
    toml.tool.huak.set_description(create_description()?);
    toml.tool.huak.set_authors(create_authors()?);
    toml.tool
        .huak
        .set_dependencies(create_dependencies("main")?);
    toml.tool
        .huak
        .set_dev_dependencies(create_dependencies("dev")?);

    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to serialize pyproject.toml"),
                2,
            ))
        }
    };

    // Serialize pyproject.toml.
    fs::write(&toml_path, string)?;

    Ok(())
}
