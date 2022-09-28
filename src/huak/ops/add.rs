use std::fs;

use crate::{
    config::pyproject::toml::Toml,
    errors::HuakError,
    package::{metadata::PyPi, python::PythonPackage},
    project::Project,
};

//(project: &Project, dependency: &str, _is_dev: bool) -> Result<(), HuakError>
pub fn add_project_dependency(
    project: &Project,
    dependency: &str,
    is_dev: bool,
) -> Result<(), HuakError> {
    let mut toml = Toml::open(&project.root.join("pyproject.toml"))?;

    // TODO: .start_with is hacky. This will fail with more data in the string
    //       (like versions).
    if toml
        .project
        .dependencies
        .iter()
        .any(|d| d.starts_with(dependency))
    {
        return Ok(());
    }

    let url = format!("https://pypi.org/pypi/{}/json", dependency);
    let res = match reqwest::blocking::get(url) {
        Ok(it) => it,
        // TODO: RequestError
        Err(e) => return Err(HuakError::AnyHowError(anyhow::format_err!(e))),
    };
    let json: PyPi = match res.json() {
        Ok(it) => it,
        // TODO: PyPIError
        Err(e) => return Err(HuakError::AnyHowError(anyhow::format_err!(e))),
    };

    // Get the version
    let version = json.info.version;
    let name = json.info.name;
    let package =
        PythonPackage::new(name.as_str(), None, Some(version.as_str()));

    let venv = match project.venv() {
        Some(v) => v,
        None => return Err(HuakError::VenvNotFound),
    };

    if venv.install_package(&package).is_err() {
        return Err(HuakError::PackageInstallFailure(dependency.to_string()));
    };

    match is_dev {
        true => toml.add_optional_dependency(dependency),
        false => toml.add_dependency(dependency),
    }

    // Serialize pyproject.toml.
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => return Err(HuakError::IOError),
    };

    if fs::write(&project.root.join("pyproject.toml"), string).is_err() {
        return Err(HuakError::IOError);
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
    fn adds_dependencies() {
        // TODO: Test optional dep.
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let mock_project_path = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_path, &directory);

        let project =
            create_mock_project(directory.join("mock-project")).unwrap();
        let toml_path = project.root.join("pyproject.toml");
        let dependency = "isort";
        let toml = Toml::open(&toml_path).unwrap();
        let had_dep = toml
            .project
            .dependencies
            .iter()
            .any(|d| d.starts_with(dependency));

        add_project_dependency(&project, dependency, false).unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let has_dep = toml
            .project
            .dependencies
            .iter()
            .any(|d| d.starts_with(dependency));

        assert!(!had_dep);
        assert!(has_dep);

        // TODO: #123 - destruction/deconstruction
    }
}
