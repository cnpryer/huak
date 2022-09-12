use std::fs;

use crate::project::Project;

use super::project_utils;

/// Create an intialized project (TODO) in an environment.
pub fn create_project(project: &Project) -> Result<(), anyhow::Error> {
    // TODO: Use available toml from manifest.
    let toml = project_utils::create_toml(project)?;
    let toml_path = project.root.join("pyproject.toml");

    if toml_path.exists() {
        return Err(anyhow::format_err!("a pyproject.toml already exists"));
    }

    // Serialize pyproject.toml.
    fs::write(&toml_path, toml.to_string()?)?;

    // Use name from the toml config.
    let name = &toml.tool.huak.name;

    // Create src subdirectory with standard project namespace.
    fs::create_dir_all(project.root.join("src"))?;
    fs::create_dir_all(project.root.join("src").join(name))?;

    // Add __init__.py to main project namespace.
    fs::write(&project.root.join("src").join(name).join("__init__.py"), "")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::test_utils::create_mock_project;

    // TODO
    #[test]
    fn creates_project() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let project = create_mock_project(directory).unwrap();

        let toml_path = project.root.join("pyproject.toml");
        let had_toml = toml_path.exists();

        create_project(&project).unwrap();

        assert!(!had_toml);
        assert!(toml_path.exists());
    }
}
