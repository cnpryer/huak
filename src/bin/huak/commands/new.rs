use std::env;
use std::fs;
use std::path::Path;

use super::utils::create_venv;
use super::utils::subcommand;
use crate::pyproject::toml::{
    create_authors, create_dependencies, create_description, create_name, create_version,
};
use clap::{arg, value_parser, ArgMatches, Command};
use huak::errors::{CliError, CliResult};
use huak::pyproject::toml::{Huak, Toml};

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

    let name = get_filename_from_path(&project_path)?;
    let toml = create_toml_from_name(name)?;

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

fn create_toml_from_name(name: String) -> Result<Toml, CliError> {
    // Create the huak spanning table of the toml file.
    let mut huak_table = Huak::new();
    huak_table.set_name(name);
    huak_table.set_version(create_version()?);
    huak_table.set_description(create_description()?);
    huak_table.set_authors(create_authors()?);
    huak_table.set_dependencies(create_dependencies("main")?);
    huak_table.set_dev_dependencies(create_dependencies("dev")?);

    let mut toml = Toml::new();
    toml.set_huak(huak_table);

    Ok(toml)
}

fn get_filename_from_path(path: &Path) -> Result<String, CliError> {
    // Attempt to convert OsStr to str.
    let name = match path.file_name() {
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

    Ok(name.unwrap().to_string())
}
