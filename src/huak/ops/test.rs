use crate::{
    errors::HuakError,
    project::{python::PythonProject, Project},
};

const MODULE: &str = "pytest";

/// Test a project using `pytest`.
pub fn test_project(project: &Project) -> Result<(), HuakError> {
    let args = [];
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(HuakError::VenvNotFound),
    };

    match venv.exec_module(MODULE, &args, &project.root) {
        Ok(_) => Ok(()),
        Err(e) => Err(HuakError::PyTestError(Box::new(e))),
    }
}
