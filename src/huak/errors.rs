use std::{io, path::PathBuf};

use thiserror::Error;

trait BinaryError {}

impl BinaryError for HuakError {}

pub type HuakResult<T> = Result<T, HuakError>;

#[derive(Error, Debug)]
pub enum HuakError {
    #[error("A Clap error occurred: {0}.")]
    ClapError(#[from] clap::Error),
    #[error("{0} already exists.")]
    DirectoryExists(PathBuf),
    #[error("Expected environment variable not found: {0}.")]
    EnvVarError(#[from] std::env::VarError),
    #[error("A pseudo-terminal error occurred: {0}.")]
    ExpectrlError(#[from] expectrl::Error),
    #[error("A formatter error occurred: {0}.")]
    FormatterError(String),
    #[error("A Git error occurred: {0}.")]
    GitError(#[from] git2::Error),
    #[error("An HTTP error occurred: {0}.")]
    HTTPError(#[from] reqwest::Error),
    #[error("A Huak configuration error occurred: {0}.")]
    HuakConfigurationError(String),
    #[error("An internal error occurred: {0}.")]
    InternalError(String),
    #[error("An IO error occurred: {0}.")]
    IOError(#[from] io::Error),
    #[error("A linter error occurred: {0}.")]
    LinterError(String),
    #[error("Failed to build the project.")]
    PyPackageBuildError,
    #[error("Failed to initialize Python package: {0}.")]
    PyPackageInitalizationError(String),
    #[error("Failed to install Python package: {0}.")]
    PyPackageInstallationError(String),
    #[error("Invalid version operator: {0}.")]
    PyPackageInvalidVersionOperator(String),
    #[error("Invalid version: {0}.")]
    PyPackageInvalidVersion(String),
    #[error("Unable to build the version specifier.")]
    PyPackageVersionSpecifierError,
    #[error(
        "The project's manifest file could not be found. \
Currently only pyproject.toml files are supported."
    )]
    PyProjectFileNotFound,
    #[error("A pyproject.toml already exists.")]
    PyProjectTomlExistsError,
    #[error("Failed to find the project's version.")]
    PyProjectVersionNotFound,
    #[error(
        "Python was not found on your operating system. Please \
install Python (https://www.python.org/)."
    )]
    PythonNotFoundError,
    #[error("No venv was found.")]
    PyVenvNotFoundError,
    #[error("A testing error occurred: {0}.")]
    TestingError(String),
    #[error("Failed to deserialize toml: {0}.")]
    TOMLDeserializationError(#[from] toml_edit::de::Error),
    #[error("Failed to serialize toml: {0}.")]
    TOMLSerializationError(#[from] toml_edit::ser::Error),
    #[error("A UTF-8 error occurred: {0}.")]
    UTF8Error(#[from] std::str::Utf8Error),
}
