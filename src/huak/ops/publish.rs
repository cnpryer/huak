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

use super::build::build_project;

const MODULE: &str = "twine";

pub fn publish_project(
    project: &Project,
    py_env: &Venv,
    installer: &Installer,
) -> HuakResult<()> {
    build_project(project, py_env, installer)?;

    if !py_env.module_path(MODULE)?.exists() {
        let package = PythonPackage::from_str(MODULE)?;
        installer.install_package(&package, py_env)?;
    }

    let runner = Runner::new()?;
    runner.run_installed_module(
        MODULE,
        &["upload", "dist/*"],
        py_env,
        Some(project.root()),
    )?;

    Ok(())
}
