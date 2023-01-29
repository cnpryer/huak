use crate::{env::venv::Venv, errors::HuakResult, project::Project};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(
    project: &Project,
    python_environment: &Venv,
) -> HuakResult<()> {
    let args = [".", "--extend-exclude", python_environment.name()?];

    python_environment.exec_module(MODULE, &args, project.root())
}
