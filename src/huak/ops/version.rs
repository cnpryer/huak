use crate::{
    errors::CliError,
    project::{python::PythonProject, Project},
};

pub fn get_project_version(project: &Project) -> Result<&str, CliError> {
    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::new(
            anyhow::format_err!("no pyproject.toml found"),
            2,
        ));
    }

    let version = project.config().project_version();

    Ok(version)
}
