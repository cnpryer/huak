use std::fs;

use crate::{config::pyproject::toml::Toml, project::Project};

pub fn create_project(project: &Project) -> Result<(), anyhow::Error> {
    let mut toml = Toml::default();
    let name = crate::utils::path::parse_filename(&project.root)?.to_string();
    toml.tool.huak.name = name.clone();

    // Serialize pyproject.toml.
    fs::write(&project.root.join("pyproject.toml"), toml.to_string()?)?;

    // Create src subdirectory with standard project namespace.
    fs::create_dir_all(project.root.join("src"))?;
    fs::create_dir_all(project.root.join("src").join(&name))?;

    // Add __init__.py to main project namespace.
    fs::write(
        &project.root.join("src").join(&name).join("__init__.py"),
        "",
    )?;

    Ok(())
}
