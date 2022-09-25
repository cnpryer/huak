use std::fs;

use crate::{errors::HuakError, project::Project};

use super::project_utils;

/// Create an initialized project (TODO) in an environment.
pub fn create_project(project: &Project) -> Result<(), HuakError> {
    // TODO: Use available toml from manifest.
    let toml = project_utils::create_toml(project)?;
    let toml_path = project.root.join("pyproject.toml");

    if toml_path.exists() {
        return Err(HuakError::AnyHowError(anyhow::format_err!(
            "A pyproject.toml already exists."
        )));
    }

    // Serialize pyproject.toml.
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => return Err(HuakError::IOError),
    };

    if fs::write(&toml_path, string).is_err() {
        return Err(HuakError::IOError);
    };

    // Use name from the toml config.
    let name = &toml.project.name;

    // Create src subdirectory with standard project namespace.
    if fs::create_dir_all(project.root.join("src")).is_err() {
        return Err(HuakError::IOError);
    };

    if fs::create_dir_all(project.root.join("src").join(name)).is_err() {
        return Err(HuakError::IOError);
    };

    // Add __init__.py to main project namespace.
    if fs::write(&project.root.join("src").join(name).join("__init__.py"), "")
        .is_err()
    {
        return Err(HuakError::IOError);
    };

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
