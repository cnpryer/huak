use crate::{
    errors::{CliError, CliResult, HuakError},
    project::{python::PythonProject, Project},
};

const MODULE: &str = "pytest";

/// Test a project using `pytest`.
pub fn test_project(project: &Project) -> CliResult<()> {
    let args = [];
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(CliError::new(HuakError::VenvNotFound, 1)),
    };

    match venv.exec_module(MODULE, &args, &project.root) {
        Ok(_) => Ok(()),
        Err(e) => {
            let code = e.status_code;
            Err(CliError::new(HuakError::PyTestError(Box::new(e)), code))
        }
    }
}
