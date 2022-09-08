use std::fs;

use crate::env::python::PythonEnvironment;
use crate::{
    config::pyproject::toml::Toml,
    project::{python::PythonProject, Project},
};

pub fn remove_project_dependency(project: &Project, dependency: &str) -> Result<(), anyhow::Error> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(anyhow::format_err!("no venv found")),
    };

    // TODO: #109
    if let Err(e) = venv.uninstall_package(dependency) {
        return Err(anyhow::format_err!(e.error.unwrap()));
    };

    let mut toml = Toml::open(&project.root.join("pyproject.toml"))?;
    toml.tool.huak.dependencies.remove(dependency);
    toml.tool.huak.dev_dependencies.remove(dependency);

    // Serialize pyproject.toml.
    fs::write(&project.root.join("pyproject.toml"), toml.to_string()?)?;

    Ok(())
}
