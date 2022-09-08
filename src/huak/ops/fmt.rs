use crate::{
    env::python::PythonEnvironment,
    errors::{CliError, CliResult},
    project::{python::PythonProject, Project},
};

/// Format Python code from the project's root.
pub fn fmt_project(project: &Project, is_check: &bool) -> CliResult {
    let from = &project.root;
    let venv = match project.venv() {
        Some(v) => v,
        None => return Err(CliError::new(anyhow::format_err!("invalid venv"), 2)),
    };
    let black_path = venv.bin_path().join("black");

    if !black_path.exists() {
        return Err(CliError::new(
            anyhow::format_err!("black is not installed"),
            2,
        ));
    }

    let black_path = crate::utils::path::as_string(&black_path)?;

    match is_check {
        true => crate::utils::command::run_command(
            black_path,
            &[".", "--line-length", "79", "--check"],
            from,
        )?,
        false => {
            crate::utils::command::run_command(black_path, &[".", "--line-length", "79"], from)?
        }
    };

    Ok(())
}
