use huak::{
    errors::CliError,
    pyproject::toml::{Huak, Toml},
};
use std::io::{self, Write};

/// Create a pyproject.toml from scratch in a directory `path`.
pub fn create() -> Result<Toml, CliError> {
    let mut toml = Toml::new();
    toml.set_huak(create_huak()?);

    Ok(toml)
}

fn create_huak() -> Result<Huak, CliError> {
    let mut name = get_string_input("Enter a name: ")?;
    name = strip_newline(&name);

    // If a name isn't entered return an error.
    if name.is_empty() {
        return Err(CliError::new(
            anyhow::format_err!("a project name is required"),
            2,
        ));
    }

    // Get the version of the project.
    let mut version = get_string_input("Please enter a version (0.0.1): ")?;
    version = strip_newline(&version);

    if version.is_empty() {
        version = "0.0.1".to_string();
    }

    // Get the description for the project.
    let mut description = get_string_input("Please enter a description (\"\"): ")?;
    description = strip_newline(&description);

    // Get the project authors.
    // TODO: Add individually.
    let mut authors = get_string_input("Please enter authors ([\"\"]): ")?;
    authors = strip_newline(&authors);

    let mut huak_table = Huak::new();
    huak_table.set_name(name);
    huak_table.set_version(version);
    huak_table.set_description(description);
    // TODO: Add individually.
    huak_table.add_author(authors);

    Ok(huak_table)
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
