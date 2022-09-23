use crate::{
    env::python::PythonEnvironment,
    errors::CliResult,
    project::{python::PythonProject, Project},
};

const MODULE: &str = "pytest";

/// Test a project using `pytest`.
pub fn test_project(project: &Project) -> CliResult<()> {
    let args = [];

    let venv = project.venv();

    venv.exec_module(MODULE, &args, &project.root)?;

    Ok(())
}
