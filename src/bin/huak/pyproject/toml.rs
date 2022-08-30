use std::io::{self, Write};

use huak::{
    errors::CliError,
    pyproject::toml::{Huak, Toml},
};

/// Create a pyproject.toml from scratch in a directory `path`.
pub fn create() -> Result<Toml, CliError> {
    let mut toml = Toml::new();
    toml.set_huak(create_huak()?);

    Ok(toml)
}

fn create_huak() -> Result<Huak, CliError> {
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

    let mut huak_table = Huak::new();
    huak_table.set_name(name);
    huak_table.set_version(version);
    huak_table.set_description(description);
    // TODO: Add individually.
    huak_table.add_author(authors);

    Ok(huak_table)
}
