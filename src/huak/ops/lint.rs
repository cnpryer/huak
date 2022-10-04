use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(project: &Project) -> HuakResult<()> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(HuakError::VenvNotFound),
    };
    let args = [".", "--extend-exclude", venv.name()?];

    venv.exec_module(MODULE, &args, &project.root)
}
