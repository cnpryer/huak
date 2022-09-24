use crate::{
    env::python::PythonEnvironment,
    env::venv::Venv,
    errors::CliResult,
    package::{metadata::PyPi, python::PythonPackage},
};

pub fn add_project_dependency(
    package: String,
    _dependency_is_dev: bool,
) -> CliResult {
    let path = format!("https://pypi.org/pypi/{}/json", package);
    let resp = reqwest::blocking::get(path)?;
    let json: PyPi = resp.json()?;
    // Get the version
    let version = json.info.version;
    let name = json.info.name;
    let mut dep = PythonPackage::new(name.clone());
    dep.name = name;
    dep.version = version;
    let venv = Venv::default();
    venv.install_package(&dep)?;
    Ok(())
}
