use std::io::{self, Write};

use crate::errors::CliError;

use self::{dependency::Dependencies, main::Main, toml::Toml};

pub mod build_system;
pub mod dependency;
pub mod main;
pub mod toml;

/// Create a pyproject.toml from scratch in a directory `path`.
pub fn create() -> Result<Toml, CliError> {
    let main = create_main()?;
    let dependencies = create_dependencies("main")?;
    let dev_dependencies = create_dependencies("dev")?;

    let mut toml = Toml::new(main);

    // Add main dependencies.
    for dependency in dependencies.list() {
        toml.add_dependency(dependency.clone(), "main");
    }

    // Add dev dependencies.
    for dependency in dev_dependencies.list() {
        toml.add_dependency(dependency.clone(), "dev");
    }

    Ok(toml)
}

fn create_main() -> Result<Main, CliError> {
    // Get the project's name.
    let mut name = String::new();

    print!("Enter a name: ");

    let _ = io::stdout().flush();

    io::stdin().read_line(&mut name)?;

    // If a name isn't entered return an error.
    if name == "\n" {
        return Err(CliError::new(
            anyhow::format_err!("a project name is required"),
            2,
        ));
    }

    // Get the version of the project.
    let mut version = String::new();

    print!("Please enter a version (0.0.1): ");

    let _ = io::stdout().flush();

    io::stdin().read_line(&mut version)?;

    // Get the description for the project.
    let mut description = String::new();

    print!("Please enter a description (\"\"): ");

    let _ = io::stdout().flush();

    io::stdin().read_line(&mut description)?;

    // Get the project authors.
    // TODO: Add individually.
    let mut authors = String::new();

    print!("Please enter authors ([\"\"]): ");

    let _ = io::stdout().flush();

    io::stdin().read_line(&mut authors)?;

    // Remove \n from strings.
    name.pop();
    version.pop();
    description.pop();
    authors.pop(); // TODO: Handle collection with future vector.

    let mut main = Main::new();
    main.set_name(name);
    main.set_version(version);
    main.set_description(description);
    // TODO: Add individually.
    main.add_author(authors);

    Ok(main)
}

/// Create either main or dev dependencies.
fn create_dependencies(kind: &str) -> Result<Dependencies, CliError> {
    // TODO
    if let "dev" = kind {
        Ok(Dependencies::new("dev"))
    } else {
        Ok(Dependencies::default())
    }
}
