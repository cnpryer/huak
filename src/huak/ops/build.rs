use std::str::FromStr;

use crate::{
    env::{
        python_environment::{PythonEnvironment, Venv},
        runner::Runner,
    },
    errors::{HuakError, HuakResult},
    package::{installer::Installer, PythonPackage},
    project::Project,
};

const MODULE: &str = "build";

pub fn build_project(
    project: &Project,
    py_env: &Venv,
    installer: &Installer,
) -> HuakResult<()> {
    if !py_env.module_path(MODULE)?.exists() {
        let package = PythonPackage::from_str("build")?;
        installer.install_package(&package, py_env)?;
    }

    let args = ["-m", MODULE];
    let runner = Runner::new()?;
    runner
        .run_installed_module("python", &args, py_env, Some(project.root()))
        .map_err(|_| HuakError::PyPackageBuildError)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::utils::test_utils::create_mock_project_full;

    use super::*;

    #[test]
    fn build() {
        let project = create_mock_project_full().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = Venv::new(&cwd.join(".venv"));
        let installer = Installer::new();

        build_project(&project, &venv, &installer).unwrap();
    }
}
