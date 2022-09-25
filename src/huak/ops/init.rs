use std::fs;

use super::project_utils;
use crate::{errors::HuakError, project::Project};

/// Initialize a project by adding a pyproject.toml to the dir.
pub fn init_project(project: &Project) -> Result<(), HuakError> {
    // Create a toml setting the name to the project dir's name.
    // TODO: Don't do this with a utility function.
    let toml = project_utils::create_toml(project)?;

    if project.root.join("pyproject.toml").exists() {
        return Err(HuakError::AnyHowError(anyhow::format_err!(
            "A pyproject.toml already exists."
        )));
    }

    // Serialize pyproject.toml.
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => return Err(HuakError::IOError),
    };

    match fs::write(&project.root.join("pyproject.toml"), string) {
        Ok(_) => Ok(()),
        Err(_) => Err(HuakError::IOError),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    // TODO
    #[test]
    fn toml() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let project = Project::from(directory).unwrap();

        init_project(&project).unwrap();

        assert!(project.root.join("pyproject.toml").exists());
    }
}
