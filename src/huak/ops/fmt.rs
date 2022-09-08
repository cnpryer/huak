use crate::{
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

/// Format Python code from the project's root.
pub fn fmt_project(project: &Project, is_check: &bool) -> CliResult {
    let venv = match project.venv() {
        Some(v) => v,
        None => return Err(CliError::new(anyhow::format_err!("invalid venv"), 2)),
    };

    match is_check {
        true => venv.exec_module(
            "black",
            &[".", "--line-length", "79", "--check"],
            &project.root,
        )?,
        false => venv.exec_module("black", &[".", "--line-length", "79"], &project.root)?,
    };

    Ok(())
}
