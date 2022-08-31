use std::{fs, path::Path};

use clap::{arg, value_parser, ArgMatches, Command};
use huak::errors::{CliError, CliResult};
use huak::pyproject::toml::{Huak, Toml};

use crate::pyproject;
use crate::utils::subcommand;

use super::utils::create_venv;

pub fn arg() -> Command<'static> {
    subcommand("new")
        .about("Create a project from scratch.")
        .arg(arg!([PATH]).id("path").value_parser(value_parser!(String)))
}

pub fn run(dir: &Path, args: &ArgMatches) -> CliResult {
    // If a path isn't passed with the `new` subcommand then use stdin.
    let target = if let Some(t) = args.get_one::<String>("path") {
        t.clone()
    } else {
        let name = &pyproject::toml::create_name()?;
        name.clone()
    };

    // TODO: Validate that target is compatible as a directory path.
    //       .is_dir() checks if the path exists. We just want to ensure that
    //       it can be created as a new directory.
    let project_path = dir.join(&target);

    // Make sure there isn't already a path we would override.
    if project_path.exists() {
        return Err(CliError::new(
            anyhow::format_err!("a path already exists"),
            2,
        ));
    }

    // Attempt to convert OsStr to str.
    let name = match project_path.file_name() {
        Some(f) => f.to_str(),
        _ => {
            return Err(CliError::new(
                anyhow::format_err!("failed to read name from path"),
                2,
            ))
        }
    };

    // If a str was failed to be parsed error.
    if name.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("failed to read name from path"),
            2,
        ));
    }

    // Create the huak spanning table of the toml file.
    let mut huak_table = Huak::new();
    huak_table.set_name(name.unwrap().to_string());
    huak_table.set_version(pyproject::toml::create_version()?);
    huak_table.set_description(pyproject::toml::create_description()?);
    huak_table.add_author(pyproject::toml::create_author()?);

    let mut toml = Toml::new();
    toml.set_huak(huak_table);

    // Create project directory.
    fs::create_dir_all(&project_path)?;

    // Attempt to prepare the serialization of pyproject.toml constructed.
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to serialize toml"),
                2,
            ))
        }
    };

    // Serialize pyproject.toml.
    fs::write(&project_path.join("pyproject.toml"), string)?;

    // Create src subdirectory with standard project namespace.
    fs::create_dir_all(dir.join(&target).join("src"))?;
    fs::create_dir_all(&project_path.join("src").join(toml.tool().huak().name()))?;

    // Add __init__.py to main project namespace.
    fs::write(
        &project_path
            .join("src")
            .join(toml.tool().huak().name())
            .join("__init__.py"),
        "",
    )?;

    // Create a .venv in the project directory using the python alias of the system.
    create_venv("python", &project_path, ".venv")?;

    Ok(())
}
