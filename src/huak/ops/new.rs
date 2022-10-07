use std::fs;

use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

/// Create an initialized project (TODO) in an environment.
pub fn create_project(project: &Project) -> HuakResult<()> {
    // TODO: Use available toml from manifest.
    let pyproject_toml = project.create_toml()?;
    let pyproject_path = project.root.join("pyproject.toml");

    if pyproject_path.exists() {
        return Err(HuakError::PyProjectTomlExists);
    }
    // bootstrap new project with lib or app template
    project.create_from_template()?;

    // Serialize pyproject.toml and write to file
    let pyproject_content = pyproject_toml.to_string()?;
    fs::write(&pyproject_path, pyproject_content)?;

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
        let directory = tempdir().unwrap().into_path();
        let project = create_mock_project(directory).unwrap();

        let toml_path = project.root.join("pyproject.toml");
        let had_toml = toml_path.exists();

        create_project(&project).unwrap();

        assert!(!had_toml);
        assert!(toml_path.exists());
    }

    #[test]
    fn create_app_project() {
        let directory = tempdir().unwrap().into_path();
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
