use std::fs;

use crate::project::Project;

use super::project_utils;

pub fn create_project(project: &Project) -> Result<(), anyhow::Error> {
    let toml = project_utils::create_toml(project)?;

    // Serialize pyproject.toml.
    fs::write(&project.root.join("pyproject.toml"), toml.to_string()?)?;

    let name = &toml.tool.huak.name;

    // Create src subdirectory with standard project namespace.
    fs::create_dir_all(project.root.join("src"))?;
    fs::create_dir_all(project.root.join("src").join(name))?;

    // Add __init__.py to main project namespace.
    fs::write(&project.root.join("src").join(name).join("__init__.py"), "")?;

    Ok(())
}
