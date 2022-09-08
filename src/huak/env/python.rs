use crate::{config::requirements::PythonPackage, errors::CliError};
use std::path::PathBuf;

pub trait PythonEnvironment {
    fn bin_path(&self) -> PathBuf;
    fn install_package(&self, package: &PythonPackage) -> Result<(), CliError>;
}
