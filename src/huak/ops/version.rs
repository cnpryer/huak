use crate::{
    errors::HuakError,
    project::{python::PythonProject, Project},
};

/// Get the version of a project.
pub fn get_project_version(project: &Project) -> Result<&str, HuakError> {
    if !project.root.join("pyproject.toml").exists() {
        return Err(HuakError::PyProjectTomlNotFound);
    }

    let version = project.config().project_version();

    Ok(version)
}
