use crate::{errors::HuakError, project::Project};

/// Get the version of a project.
pub fn get_project_version(project: &Project) -> Result<&str, HuakError> {
    if !project.root().join("pyproject.toml").exists() {
        return Err(HuakError::PyProjectFileNotFound);
    }

    let version = project.project_file.project_version();

    match version {
        Some(version) => Ok(version),
        None => Err(HuakError::PyProjectVersionNotFound),
    }
}
