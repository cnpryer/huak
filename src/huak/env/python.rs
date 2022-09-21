use crate::{errors::CliError, package::python::PythonPackage};
use std::path::{Path, PathBuf};

pub trait PythonEnvironment {
    fn bin_path(&self) -> PathBuf;
    fn module_path(&self, module: &str) -> Result<PathBuf, anyhow::Error>;
    fn exec_module(
        &self,
        module: &str,
        args: &[&str],
        from: &Path,
    ) -> Result<(), CliError>;
    fn install_package(&self, package: &PythonPackage) -> Result<(), CliError>;
    fn uninstall_package(&self, name: &str) -> Result<(), CliError>;
}
