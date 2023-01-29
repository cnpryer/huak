use crate::{
    env::venv::Venv,
    errors::{HuakError, HuakResult},
};

use super::{index::PackageIndexData, PythonPackage};

#[derive(Default)]
pub struct PythonPackageInstaller {
    pub ignore_cache: bool,
}

impl PythonPackageInstaller {
    pub fn new() -> Self {
        PythonPackageInstaller {
            ..Default::default()
        }
    }

    pub fn install_package(
        &self,
        package: &PythonPackage,
        python_environment: &Venv,
    ) -> Result<(), HuakError> {
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

        python_environment.install_package(&package).map_err(|_| {
            HuakError::PyPackageInstallFailure(package.to_string())
        })?;

        Ok(())
    }

    pub fn install_packages(
        &self,
        packages: &Vec<PythonPackage>,
        python_environment: &Venv,
    ) -> HuakResult<()> {
        for package in packages {
            python_environment.install_package(package)?;
        }

        Ok(())
    }

    pub fn last_installed_package(
        &self,
    ) -> Result<Option<PythonPackage>, HuakError> {
        Ok(None)
    }

    pub fn search_installed(
        &self,
        _package: &PythonPackage,
    ) -> Result<Option<Vec<PythonPackage>>, HuakError> {
        Ok(None)
    }
}

fn get_package_index_data(
    package: &PythonPackage,
) -> HuakResult<PackageIndexData> {
    let name = &package.name;
    let url = format!("https://pypi.org/pypi/{name}/json");
    let res = match reqwest::blocking::get(url) {
        Ok(it) => it,
        // TODO: RequestError
        Err(e) => return Err(HuakError::InternalError(e.to_string())),
    };
    match res.json() {
        Ok(it) => Ok(it),
        // TODO: PyPIError
        Err(e) => Err(HuakError::InternalError(e.to_string())),
    }
}
