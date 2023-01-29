use crate::{env::venv::Venv, errors::HuakResult, project::Project};

const MODULE: &str = "pytest";

/// Test a project using `pytest`.
pub fn test_project(
    project: &Project,
    python_environment: &Venv,
) -> HuakResult<()> {
    python_environment.exec_module(MODULE, &[], project.root())
}
