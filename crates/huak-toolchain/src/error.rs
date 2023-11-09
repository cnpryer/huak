use std::path::PathBuf;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("a problem occurred attempting to link a file: {0}")]
    FileLinkFailure(String),
    #[error("a file could not be found: {0}")]
    FileNotFound(PathBuf),
    #[error("a problem occurred due to an invalid toolchain: {0}")]
    InvalidToolchain(String),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("a problem occurred attempting to parse a channel: {0}")]
    ParseChannelError(String),
    #[error("a problem occurred attempting to install python {0}")]
    PythonInstallationError(String),
    #[error("{0}")]
    PythonManagerError(#[from] huak_python_manager::Error),
    #[error("a local tool could not be found: {0}")]
    LocalToolNotFound(PathBuf),
    #[error("a toolchain already exists: {0}")]
    LocalToolchainExistsError(PathBuf),
    #[error("{0}")]
    TOMLEditError(#[from] toml_edit::TomlError),
    #[error("a problem with utf-8 parsing occurred: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}
