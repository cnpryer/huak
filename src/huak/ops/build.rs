use crate::{
    env::venv::Venv, errors::HuakError, package::python::PythonPackage,
    project::Project,
};

const MODULE: &str = "build";

pub fn build_project(project: &Project) -> Result<(), HuakError> {
    let venv = &Venv::from_path(project.root())?;

    let package = PythonPackage::from("build")?;

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

        build_project(&project).unwrap();
    }
}
