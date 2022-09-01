use huak::{
    errors::CliError,
    pyproject::toml::{Huak, Toml},
    Dependency,
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
    let name = get_string_input("Enter a name: ")?;

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

    if version.is_empty() {
        version = "0.0.1".to_string();
    }

    Ok(version)
}

pub fn create_description() -> Result<String, io::Error> {
    // Get the description for the project.
    let description = get_string_input("Please enter a description (\"\"): ")?;

    Ok(description)
}

pub fn create_authors() -> Result<Vec<String>, io::Error> {
    let mut authors = Vec::new();

    loop {
        let author = create_author()?;

        match author.is_empty() {
            true => break,
            false => authors.push(author),
        }
    }

    Ok(authors)
}

pub fn create_author() -> Result<String, io::Error> {
    let author = get_string_input("Please enter an author (\"\"): ")?;

    Ok(author)
}

pub fn create_dependencies(kind: &str) -> Result<Vec<Dependency>, CliError> {
    let mut dependencies = Vec::new();

    loop {
        let dependency = &create_dependency(kind)?;

        match dependency {
            Some(dep) => dependencies.push(dep.clone()),
            _ => break,
        }
    }

    Ok(dependencies)
}

pub fn create_dependency(kind: &str) -> Result<Option<Dependency>, CliError> {
    let message = format!("Enter a {} dependency (package): ", kind);
    let name = get_string_input(&message)?;

    if name.is_empty() {
        return Ok(None);
    }

    let message = format!("Enter a version for {} (x.x.x): ", name);
    let version = get_string_input(&message)?;

    Ok(Some(Dependency { name, version }))
}

fn get_string_input(message: &str) -> Result<String, io::Error> {
    let mut res = String::new();

    print!("{}", message);

    let _ = io::stdout().flush();

    io::stdin().read_line(&mut res)?;

    res = strip_newline(&res);

    Ok(res)
}

fn strip_newline(string: &str) -> String {
    let mut new_string = string.to_string();

    if new_string.ends_with('\n') {
        new_string.pop();
    }

    new_string
}
