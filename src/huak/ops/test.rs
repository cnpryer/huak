use crate::{env::venv::Venv, errors::HuakResult, project::Project};

const MODULE: &str = "pytest";

/// Test a project using `pytest`.
pub fn test_project(project: &Project) -> HuakResult<()> {
    let args = [];
    let venv = &Venv::from_path(project.root())?;

    // TODO: not sure if the PyTestError was something you wanted to override
    // the internal HuakError, so leaving as comment for now.
    // match venv.exec_module(MODULE, &args, &project.root) {
    //     Ok(_) => Ok(()),
    //     Err(e) => Err(HuakError::PyTestError(Box::new(e))),
    // }
    venv.exec_module(MODULE, &args, project.root())
}
