use std::env;
use std::fs;

use super::utils::create_venv;
use super::utils::subcommand;
use crate::pyproject::toml::create_authors;
use crate::pyproject::toml::create_dependencies;
use crate::pyproject::toml::create_description;
use crate::pyproject::toml::create_name;
use crate::pyproject::toml::create_version;
use clap::{arg, value_parser, ArgMatches, Command};
use huak::errors::{CliError, CliResult};
use huak::pyproject::toml::Toml;

pub fn arg() -> Command<'static> {
    subcommand("new")
        .about("Create a project from scratch.")
        .arg(arg!([PATH]).id("path").value_parser(value_parser!(String)))
}

pub fn run(args: &ArgMatches) -> CliResult {
    // This command runs from the current working directory
    // Each command's behavior is triggered from the context of the cwd.
    let cwd_buff = env::current_dir()?;
    let dir = cwd_buff.as_path();

    // If a path isn't passed with the `new` subcommand then use stdin.
    let target = if let Some(t) = args.get_one::<String>("path") {
        t.clone()
    } else {
        let name = &create_name()?;
        name.clone()
    };

    // TODO: Validate that target is compatible as a directory path.
    //       .is_dir() checks if the path exists. We just want to ensure that
    //       it can be created as a new directory.
    let project_path = dir.join(&target);

    // Make sure there isn't already a path we would override.
    if project_path.exists() {
        return Err(CliError::new(
            anyhow::format_err!("a directory already exists"),
            2,
        ));
    }

    let name = huak::utils::get_filename_from_path(&project_path)?;
    let mut toml = Toml::new();

    toml.tool.huak.set_name(name);
    toml.tool.huak.set_version(create_version()?);
    toml.tool.huak.set_description(create_description()?);
    toml.tool.huak.set_authors(create_authors()?);
    toml.tool
        .huak
        .set_dependencies(create_dependencies("main")?);
    toml.tool.huak.set_dependencies(create_dependencies("dev")?);

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
