use crate::{
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

pub fn test_project(project: &Project) -> CliResult {
    let module = "pytest";
    let args = [];

    let venv = match project.venv() {
        Some(v) => v,
        _ => {
            return Err(CliError::new(anyhow::format_err!("no venv found"), 2))
        }
    };

    venv.exec_module(module, &args, &project.root)?;

    Ok(())
}
