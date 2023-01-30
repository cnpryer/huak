use std::{io, path::PathBuf};

use thiserror::Error;

trait BinaryError {}

impl BinaryError for HuakError {}

pub type HuakResult<T> = Result<T, HuakError>;

#[derive(Error, Debug)]
pub enum HuakError {
    #[error("a problem with argument parsing occurred: {0}")]
    ClapError(#[from] clap::Error),
    #[error("a directory already exists: {0}")]
    DirectoryExists(PathBuf),
    #[error("a problem with the environment occurred: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("a problem with the pseudo-terminal occurred: {0}")]
    ExpectrlError(#[from] expectrl::Error),
    #[error("a problem with the formatter occurred: {0}")]
    FormatterError(String),
    #[error("a problem with git occurred: {0}")]
    GitError(#[from] git2::Error),
    #[error("a problem with http occurred: {0}")]
    HTTPError(#[from] reqwest::Error),
    #[error("a problem with huak configuration occurred: {0}")]
    HuakConfigurationError(String),
    #[error("a problem with huak's internals occurred: {0}")]
    InternalError(String),
    #[error("a problem with io occurred: {0}")]
    IOError(#[from] io::Error),
    #[error("a problem with the linter occurred: {0}")]
    LinterError(String),
    #[error("a problem with building the project occurred")]
    PyPackageBuildError,
    #[error("a problem with the package index occurred: {0}")]
    PyPackageIndexError(String),
    #[error("a problem with package initialization occurred: {0}")]
    PyPackageInitalizationError(String),
    #[error("a problem with package installation occurred: {0}")]
    PyPackageInstallationError(String),
    #[error("a problem with the package version operator occurred: {0}")]
    PyPackageInvalidVersionOperator(String),
    #[error("a problem with the package version occurred: {0}")]
    PyPackageInvalidVersion(String),
    #[error("a problem with the package version specifier occurred")]
    PyPackageVersionSpecifierError,
    #[error("a project file could not be found")]
    PyProjectFileNotFound,
    #[error("a pyproject.toml already exists")]
    PyProjectTomlExistsError,
    #[error("a problem with locating the project's version number occurred")]
    PyProjectVersionNotFound,
    #[error("a python interpreter could not be found")]
    PythonNotFoundError,
    #[error("a venv could not be found")]
    PyVenvNotFoundError,
    #[error("a problem with the test utility occurred: {0}")]
    TestingError(String),
    #[error("a problem with toml deserialization occurred: {0}")]
    TOMLDeserializationError(#[from] toml_edit::de::Error),
    #[error("a problem with toml serialization occurred {0}")]
    TOMLSerializationError(#[from] toml_edit::ser::Error),
    #[error("a problem with utf-8 parsing occurred: {0}")]
    UTF8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    WrappedCommandError(String),
}
