use crate::{
    config::python::PythonConfig,
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

pub fn install_project_dependencies(project: &Project) -> CliResult {
    let venv = match project.venv() {
        Some(v) => v,
        _ => {
            return Err(CliError::new(
                anyhow::format_err!("failed to setup venv"),
                2,
            ))
        }
    };

    venv.create()?;

    if !project.root.join("pyproject.toml").exists() {
        return Err(CliError::new(
            anyhow::format_err!("no pyproject.toml found"),
            2,
        ));
    }

    for dependency in project.config().dependencies() {
        venv.install_package(dependency)?;
    }

    for dependency in project.config().dev_dependencies() {
        venv.install_package(dependency)?;
    }

    Ok(())
}
