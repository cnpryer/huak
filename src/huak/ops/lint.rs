use crate::{
    env::python::PythonEnvironment,
    errors::CliResult,
    project::{python::PythonProject, Project},
};

/// Lint the project using `flake8`.
pub fn lint_project(project: &Project) -> CliResult {
    // Use the `flake` module for now.
    let module = "flake8";

    // TODO
    let args = ["--ignore", "E203,W503", "--exclude", project.venv().name()?];

    project.venv().exec_module(module, &args, &project.root)?;

    Ok(())
}
