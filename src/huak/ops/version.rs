use crate::{
    errors::CliError,
    project::{python::PythonProject, Project},
};

/// Get the version of a project.
pub fn get_project_version(project: &Project) -> Result<&str, CliError> {
    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::from(anyhow::format_err!(
            "No pyproject.toml found"
        )));
    }

    let version = project.config().project_version();

    Ok(version)
}
