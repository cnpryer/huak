use crate::{env::venv::Venv, errors::HuakResult, project::Project};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(project: &Project) -> HuakResult<()> {
    let venv = &Venv::from_path(project.root())?;
    let args = [".", "--extend-exclude", venv.name()?];

    venv.exec_module(MODULE, &args, project.root())
}
