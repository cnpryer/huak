use crate::{config::requirements::PythonPackage, errors::CliError};
use std::path::{Path, PathBuf};

pub trait PythonEnvironment {
    fn bin_path(&self) -> PathBuf;
    fn exec_module(&self, module: &str, args: &[&str], from: &Path) -> Result<(), CliError>;
    fn install_package(&self, package: &PythonPackage) -> Result<(), CliError>;
    fn uninstall_package(&self, name: &str) -> Result<(), CliError>;
}
