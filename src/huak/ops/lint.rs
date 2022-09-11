use crate::{
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

pub fn lint_project(project: &Project) -> CliResult {
    let module = "flake8";

    let venv = match project.venv() {
        Some(v) => v,
        _ => {
            return Err(CliError::new(
                anyhow::format_err!("failed to locate the project's venv"),
                2,
            ))
        }
    };

    let args = ["--ignore", "E203,W503", "--exclude", venv.name()?];

    venv.exec_module(module, &args, &project.root)?;

    Ok(())
}
