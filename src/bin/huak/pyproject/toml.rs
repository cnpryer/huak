use huak::{
    errors::CliError,
    pyproject::toml::{Huak, Toml},
};
use std::io::{self, Write};

/// Create a pyproject.toml from scratch in a directory `path`.
#[allow(dead_code)]
pub fn create() -> Result<Toml, CliError> {
    let mut toml = Toml::new();
    toml.set_huak(create_huak()?);

    Ok(toml)
}

#[allow(dead_code)]
/// Create huak table for toml.
fn create_huak() -> Result<Huak, CliError> {
    let mut huak_table = Huak::new();

    huak_table.set_name(create_name()?);
    huak_table.set_version(create_version()?);
    huak_table.set_description(create_description()?);
    // TODO: Add many individually.
    huak_table.add_author(create_author()?);

    Ok(huak_table)
}

pub fn create_name() -> Result<String, CliError> {
    let mut name = get_string_input("Enter a name: ")?;
    name = strip_newline(&name);

    // If a name isn't entered return an error.
    if name.is_empty() {
        return Err(CliError::new(
            anyhow::format_err!("a project name is required"),
            2,
        ));
    }

    Ok(name)
}

pub fn create_version() -> Result<String, io::Error> {
    // Get the version of the project.
    let mut version = get_string_input("Please enter a version (0.0.1): ")?;
    version = strip_newline(&version);

    if version.is_empty() {
        version = "0.0.1".to_string();
    }

    Ok(version)
}

pub fn create_description() -> Result<String, io::Error> {
    // Get the description for the project.
    let mut description = get_string_input("Please enter a description (\"\"): ")?;
    description = strip_newline(&description);

    Ok(description)
}

pub fn create_author() -> Result<String, io::Error> {
    // Get the project authors.
    let mut authors = get_string_input("Please enter authors ([\"\"]): ")?;
    authors = strip_newline(&authors);

    Ok(authors)
}

fn get_string_input(message: &str) -> Result<String, io::Error> {
    let mut res = String::new();

    print!("{}", message);

    let _ = io::stdout().flush();

    io::stdin().read_line(&mut res)?;

    Ok(res)
}

fn strip_newline(string: &str) -> String {
    let mut new_string = string.to_string();

    if new_string.ends_with('\n') {
        new_string.pop();
    }

    new_string
}
