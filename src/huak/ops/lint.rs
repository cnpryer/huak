use crate::{
    errors::{CliError, CliResult, HuakError},
    project::{python::PythonProject, Project},
};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(project: &Project) -> CliResult<()> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(CliError::new(HuakError::VenvNotFound, 1)),
    };
    let args = [".", "--extend-exclude", venv.name()?];

    match venv.exec_module(MODULE, &args, &project.root) {
        Err(e) => {
            let code = e.status_code;
            Err(CliError::new(HuakError::RuffError(Box::new(e)), code))
        }
        Ok(_) => Ok(()),
    }
}
