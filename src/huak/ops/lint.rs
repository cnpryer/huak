use crate::{
    env::python::PythonEnvironment,
    errors::CliResult,
    project::{python::PythonProject, Project},
};

/// Lint the project using `ruff`.
pub fn lint_project(project: &Project) -> CliResult {
    // Use the `ruff` module for now.
    let module = "ruff";

    // TODO
    let args = [".", "--exclude", project.venv().name()?];

    project.venv().exec_module(module, &args, &project.root)?;

    Ok(())
}
