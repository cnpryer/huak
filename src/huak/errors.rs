use std::{io, path::PathBuf};

use thiserror::Error;

trait BinaryError {}

impl BinaryError for HuakError {}

pub type HuakResult<T> = Result<T, HuakError>;

#[derive(Error, Debug)]
pub enum HuakError {
    #[error(
        "This feature is not implemented. See https://github.com/cnpryer/huak/milestones."
    )]
    NotImplemented,
    #[error("Some arguments were missing.")]
    MissingArguments,
    #[error(
        "An unknown error occurred: {0}. Please file a bug report at \
        https://github.com/cnpryer/huak/issues/new?\
        assignees=&labels=bug&template=BUG_REPORT.md&title="
    )]
    UnknownError(String),
    #[error("An IO error occurred: {0}.")]
    IOError(#[from] io::Error),
    #[error(
        "This is an unknown command. Please rerun with the `--help` flag."
    )]
    UnknownCommand,
    #[error("{0}")]
    ClapError(#[from] clap::Error),
    #[error("{0}")]
    ConfigurationError(String),
    #[error("{0} already exists and may not be empty!")]
    DirectoryExists(PathBuf),
    #[error("An HTTP error occurred: {0}.")]
    HttpError(#[from] reqwest::Error),
    #[error("An error occurred while parsing bytes to a string: {0}.")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    InternalError(String),
    // TODO: make RuffError, PyBlackError, PyTestError, etc, take in whatever you
    // feel makes the most sense. I have them as String for now, since I can't find
    // usages anywhere and can't tell what they should derive from.
    #[error("Ruff Error: {0}")]
    RuffError(String),
    #[error("Black Error: {0}")]
    PyBlackError(String),
    #[error("Pytest Error: {0}")]
    PyTestError(String),
    #[error(
        "Python was not found on your operating system. Please \
        install Python at https://www.python.org/."
    )]
    PythonNotFound,
    #[error("No venv was found.")]
    VenvNotFound,
    #[error("Expected env var not found.")]
    EnvVarError(#[from] std::env::VarError),
    #[error("A pyproject.toml could not be found.")]
    PyProjectTomlNotFound, // TODO: Manifest
    #[error("Failed to install Python package: {0}.")]
    PyPackageInstallFailure(String),
    #[error("A pyproject.toml already exists.")]
    PyProjectTomlExists,
    #[error("Failed to init Python package: {0}.")]
    PyPackageInitError(String),
    #[error("Failed to deserialize toml: {0}.")]
    TomlDeserializeError(#[from] toml_edit::de::Error),
    #[error("Failed to serialize toml: {0}.")]
    TomlSerializeError(#[from] toml_edit::ser::Error),
    #[error("Invalid Python package version operator: {0}.")]
    InvalidPyPackageVersionOp(String),
    #[error("Failed to build the project.")]
    BuildFailure,
    #[error("Failed to find the project's version.")]
    VersionNotFound,
    #[error("Invalid version operator {0}.")]
    PyPackageInvalidOperator(String),
    #[error("Invalid version {0}.")]
    PyPackageInvalidVersion(String),
    #[error("Unable to build the version specifier.")]
    PyPackageVersionSpecifierError,
    #[error("Error related to pseudo-terminal: {0}.")]
    ExpectrlError(#[from] expectrl::Error),
    #[error("Project name not found.")]
    ProjectNameNotFound,
    #[error("Git error: {0}.")]
    GitError(#[from] git2::Error),
}
