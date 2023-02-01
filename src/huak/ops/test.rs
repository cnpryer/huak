use std::str::FromStr;

use crate::{
    env::{runner::Runner, venv::Venv},
    errors::HuakResult,
    package::{installer::Installer, PythonPackage},
    project::Project,
};

const MODULE: &str = "pytest";

/// Test a project using `pytest`.
pub fn test_project(
    project: &Project,
    py_env: &Venv,
    installer: &Installer,
) -> HuakResult<()> {
    if !py_env.module_path(MODULE)?.exists() {
        let package = PythonPackage::from_str(MODULE)?;
        installer.install_package(&package, py_env)?;
    }

    let runner = Runner::new()?;
    runner.run_installed_module(MODULE, &[], py_env, Some(project.root()))
}
