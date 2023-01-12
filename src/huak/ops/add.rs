use std::fs;

use crate::{
    config::pyproject::toml::Toml,
    env::venv::Venv,
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
    let mut toml = Toml::open(&project.root().join("pyproject.toml"))?;

    // TODO: .start_with is hacky. This will fail with more data in the string
    //       (like versions).
    if toml
        .project
        .dependencies
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|d| d.starts_with(dependency))
    {
        return Ok(());
    }

    let url = format!("https://pypi.org/pypi/{}/json", dependency);
    let res = match reqwest::blocking::get(url) {
        Ok(it) => it,
        // TODO: RequestError
        Err(e) => return Err(HuakError::InternalError(e.to_string())),
    };
    let json: PyPi = match res.json() {
        Ok(it) => it,
        // TODO: PyPIError
        Err(e) => return Err(HuakError::InternalError(e.to_string())),
    };

    // Get the version
    let version = json.info.version;
    let name = json.info.name;
    let package =
        PythonPackage::new(name.as_str(), None, Some(version.as_str()))?;

    let venv = &Venv::from_path(project.root())?;

    let dep = package.string();

    venv.install_package(&package)
        .map_err(|_| HuakError::PyPackageInstallFailure(dep.clone()))?;

    match is_dev {
        true => toml.add_optional_dependency("dev", dep),
        false => toml.add_dependency(dep),
    }

    // Serialize pyproject.toml.
    let string = toml.to_string()?;
    fs::write(project.root().join("pyproject.toml"), string)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::test_utils::create_mock_project_full;

    #[test]
    fn adds_dependencies() {
        // TODO: Test optional dep.
        let project = create_mock_project_full().unwrap();
        let toml_path = project.root().join("pyproject.toml");
        let dependency = "isort";
        let toml = Toml::open(&toml_path).unwrap();
        let had_dep = toml
            .project
            .dependencies
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .any(|d| d.starts_with(dependency));

        add_project_dependency(&project, dependency, false).unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let has_dep = toml
            .project
            .dependencies
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .any(|d| d.starts_with(dependency));

        assert!(!had_dep);
        assert!(has_dep);

        // TODO: #123 - destruction/deconstruction
    }
}
