use crate::{
    env::python::PythonEnvironment,
    errors::CliResult,
    project::{python::PythonProject, Project},
};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(project: &Project) -> CliResult {
    let args = [".", "--extend-exclude", project.venv().name()?];

    project.venv().exec_module(MODULE, &args, &project.root)?;

    Ok(())
}
