use std::fs;

use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

use super::project_utils;

/// Create an initialized project (TODO) in an environment.
pub fn create_project(project: &Project) -> HuakResult<()> {
    // TODO: Use available toml from manifest.
    let toml = project_utils::create_toml(project)?;
    let toml_path = project.root.join("pyproject.toml");

    if toml_path.exists() {
        return Err(HuakError::PyProjectTomlExists);
    }

    // Serialize pyproject.toml.
    let string = toml.to_string()?;
    fs::write(&toml_path, string)?;

    // Use name from the toml config.
    let name = &toml.project.name;

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

    use crate::{
        config::pyproject::toml::Toml, project::ProjectType,
        utils::test_utils::create_mock_project,
    };

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

    #[test]
    fn create_app_project() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let project = Project::new(directory, ProjectType::Application);
        let toml_path = project.root.join("pyproject.toml");

        create_project(&project).unwrap();
        let toml = Toml::open(&toml_path).unwrap();

        assert!(toml.project.scripts.is_some());
        assert_eq!(
            toml.project.scripts.unwrap()[&toml.project.name],
            format!("{}:run", toml.project.name)
        );
    }
}
