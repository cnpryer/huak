use crate::{
    errors::{CliError, CliResult, HuakError},
    project::{python::PythonProject, Project},
};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(project: &Project) -> CliResult<()> {
    let args = [".", "--extend-exclude", project.venv().name()?];

    match project.venv().exec_module(MODULE, &args, &project.root) {
        Err(e) => {
            let code = e.status_code;
            Err(CliError::new(HuakError::PyBlackError(Box::new(e)), code))
        }
        Ok(_) => Ok(()),
    }
}
