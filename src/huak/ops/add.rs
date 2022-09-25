use crate::{
    env::venv::Venv,
    errors::{CliResult, HuakError},
    package::{metadata::PyPi, python::PythonPackage}, project::{Project, python::PythonProject},
};

//(project: &Project, dependency: &str, _is_dev: bool) -> Result<(), HuakError>
pub fn add_project_dependency(project: &Project, dependency: &str, _is_dev: bool) -> Result<(), HuakError> {
    let url = format!("https://pypi.org/pypi/{}/json", dependency);
    let res = match reqwest::blocking::get(url) {
        Ok(it) => it,
        Err(e) => return Err(HuakError::AnyHowError(anyhow::format_err!(e))),
    };
    let json: PyPi = match res.json() {
        Ok(it) => it,
        Err(e) => return Err(HuakError::AnyHowError(anyhow::format_err!(e))),
    };
    
    // Get the version
    let version = json.info.version;
    let name = json.info.name;
    let package = PythonPackage::new(name.as_str(), None, Some(version.as_str()));
    
    match project.venv() {
        Some(v) => v.install_package(&package),
        _ => return Err(HuakError::VenvNotFound),
    };

    Ok(())
}
