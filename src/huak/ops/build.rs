use crate::{
    errors::HuakError, package::python::PythonPackage, project::Project,
};

const MODULE: &str = "build";

pub fn build_project(project: &Project) -> Result<(), HuakError> {
    let venv = match project.venv() {
        Some(it) => it,
        None => return Err(HuakError::VenvNotFound),
    };

    let package = PythonPackage::from("build".to_string())?;

    if venv.install_package(&package).is_err() {
        return Err(HuakError::PyPackageInstallFailure("build".to_string()));
    };

    let args = ["-m", MODULE];
    if venv.exec_module("python", &args, &project.root).is_err() {
        return Err(HuakError::BuildFailure);
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::{
        path::copy_dir,
        test_utils::{create_mock_project, get_resource_dir},
    };

    #[test]
    fn build() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let mock_project_dir = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_dir, &directory);

        let project_path = directory.join("mock-project");
        let project = create_mock_project(project_path.clone()).unwrap();

        build_project(&project).unwrap();
    }
}
