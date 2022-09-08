use crate::{
    config::python::PythonConfig,
    env::{python::PythonEnvironment, venv::Venv},
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

pub fn install_project_dependencies(project: &Project) -> CliResult {
    // TODO: Doing this venv handling seems hacky.
    let mut venv = &Venv::new(project.root.join(".venv"));
    if let Some(v) = project.venv() {
        venv = v
    } else {
        venv.create()?;
    }

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
