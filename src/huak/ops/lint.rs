use std::str::FromStr;

use crate::{
    env::{
        python_environment::{PythonEnvironment, Venv},
        runner::Runner,
    },
    errors::HuakResult,
    package::{installer::Installer, PythonPackage},
    project::Project,
};

const MODULE: &str = "ruff";

/// Lint the project from its root.
pub fn lint_project(
    project: &Project,
    py_env: &Venv,
    installer: &Installer,
) -> HuakResult<()> {
    if !py_env.module_path(MODULE)?.exists() {
        let package = PythonPackage::from_str(MODULE)?;
        installer.install_package(&package, py_env)?;
    }

    let args = [".", "--extend-exclude", py_env.name()?];
    let runner = Runner::new()?;

    runner.run_installed_module(MODULE, &args, py_env, Some(project.root()))
}
