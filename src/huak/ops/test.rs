use crate::{
    env::python::PythonEnvironment,
    errors::CliResult,
    project::{python::PythonProject, Project},
};

/// Test a project using `pytest`.
pub fn test_project(project: &Project) -> CliResult {
    let module = "pytest";
    let args = [];

    let venv = project.venv();

    venv.exec_module(module, &args, &project.root)?;

    Ok(())
}
