use crate::{
    env::{python_environment::Venv, runner::Runner},
    errors::{HuakError, HuakResult},
};

use super::{index::PackageIndexData, PythonPackage};

#[derive(Default)]
pub struct Installer {
    pub ignore_cache: bool,
}

impl Installer {
    pub fn new() -> Self {
        Installer {
            ..Default::default()
        }
    }

    pub fn install_package(
        &self,
        package: &PythonPackage,
        py_env: &Venv,
    ) -> HuakResult<()> {
        let package = match self.search_installed(package)? {
            Some(_) => todo!(),
            None => {
                let package_metadata = get_package_index_data(package)?;

                // Get the version
                let version = package_metadata.info.version;
                let name = package_metadata.info.name;
                let package = PythonPackage::from_str_parts(
                    name.as_str(),
                    None,
                    version.as_str(),
                )?;
                package
            }
        };

        install_package_with_pip(&package, py_env).map_err(|_| {
            HuakError::PyPackageInstallationError(package.to_string())
        })?;

        Ok(())
    }

    pub fn install_packages(
        &self,
        packages: &Vec<PythonPackage>,
        py_env: &Venv,
    ) -> HuakResult<()> {
        for package in packages {
            install_package_with_pip(package, py_env)?;
        }

        Ok(())
    }

    pub fn last_installed_package(&self) -> HuakResult<Option<PythonPackage>> {
        Ok(None)
    }

    pub fn search_installed(
        &self,
        _package: &PythonPackage,
    ) -> HuakResult<Option<Vec<PythonPackage>>> {
        Ok(None)
    }

    pub fn uninstall_package(
        &self,
        name: &str,
        py_env: &Venv,
    ) -> HuakResult<()> {
        uninstall_package_with_pip(name, py_env)
    }
}

fn get_package_index_data(
    package: &PythonPackage,
) -> HuakResult<PackageIndexData> {
    let name = &package.name;
    let url = format!("https://pypi.org/pypi/{name}/json");
    let res = match reqwest::blocking::get(url) {
        Ok(it) => it,
        Err(e) => return Err(HuakError::PyPackageIndexError(e.to_string())),
    };
    match res.json() {
        Ok(it) => Ok(it),
        // TODO: PyPIError
        Err(e) => Err(HuakError::InternalError(e.to_string())),
    }
}

/// Install a Python package to a python environment.
fn install_package_with_pip(
    package: &PythonPackage,
    py_env: &Venv,
) -> HuakResult<()> {
    let runner = Runner::new()?;
    runner.run_installed_module(
        "pip",
        &["install", &package.name],
        py_env,
        None,
    )?;

    Ok(())
}

/// Uninstall a dependency from a python environment.
fn uninstall_package_with_pip(name: &str, py_env: &Venv) -> HuakResult<()> {
    let runner = Runner::new()?;
    runner.run_installed_module(
        "pip",
        &["uninstall", name, "-y"],
        py_env,
        None,
    )?;

    Ok(())
}
