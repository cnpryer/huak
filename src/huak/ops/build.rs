use std::str::FromStr;

use crate::{
    env::venv::Venv, errors::HuakError, package::PythonPackage,
    project::Project,
};

const MODULE: &str = "build";

pub fn build_project(project: &Project, venv: &Venv) -> Result<(), HuakError> {
    let package = PythonPackage::from_str("build")?;

    venv.install_package(&package)
        .map_err(|_| HuakError::PyPackageInstallFailure("build".to_string()))?;

    let args = ["-m", MODULE];
    venv.exec_module("python", &args, project.root())
        .map_err(|_| HuakError::BuildFailure)?;

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
        let venv = Venv::new(cwd.join(".venv"));

        build_project(&project, &venv).unwrap();
    }
}
