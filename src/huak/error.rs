use crate::sys;
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

pub type HuakResult<T> = Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("a problem with argument parsing occurred: {0}")]
    ClapError(#[from] clap::Error),
    #[error("a directory already exists: {0}")]
    DirectoryExists(PathBuf),
    #[error("a problem with the environment occurred: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("a problem with git occurred: {0}")]
    GitError(#[from] git2::Error),
    #[error("a problem occurred with the glob package: {0}")]
    GlobError(#[from] glob::GlobError),
    #[error("a problem occurred with a glob pattern: {0}")]
    GlobPatternError(#[from] glob::PatternError),
    #[error("a problem with huak configuration occurred: {0}")]
    HuakConfigurationError(String),
    #[error("a problem with huak's internals occurred: {0}")]
    InternalError(String),
    #[error("a version number could not be parsed: {0}")]
    InvalidVersionString(String),
    #[error("a problem occurred with json deserialization: {0}")]
    JSONSerdeError(#[from] serde_json::Error),
    #[error("a problem with io occurred: {0}")]
    IOError(#[from] io::Error),
    #[error("a problem occurred with PEP440 parsing: {0}")]
    PEP440Error(#[from] pep440_rs::Pep440Error),
    #[error("a problem occurred with PEP508 parsing: {0}")]
    PEP508Error(#[from] pep508_rs::Pep508Error),
    #[error("a metadata file already exists")]
    MetadataFileFound,
    #[error("a metadata file could not be found")]
    MetadataFileNotFound,
    #[error("a package version could not be found")]
    PackageVersionNotFound,
    #[error("a project already exists")]
    ProjectFound,
    #[error("a python interpreter could not be found")]
    PythonNotFound,
    #[error("a python environment could not be found")]
    PythonEnvironmentNotFound,
    #[error("a regex error occurred: {0}")]
    RegexError(#[from] regex::Error),
    #[error("a subprocess exited with {0}")]
    SubprocessFailure(sys::SubprocessError),
    #[error("a problem with toml deserialization occurred: {0}")]
    TOMLDeserializationError(#[from] toml::de::Error),
    #[error("a problem with toml serialization occurred {0}")]
    TOMLSerializationError(#[from] toml::ser::Error),
    #[error("a problem with toml deserialization occurred: {0}")]
    TOMLEditDeserializationError(#[from] toml_edit::de::Error),
    #[error("a problem with toml serialization occurred {0}")]
    TOMLEditSerializationError(#[from] toml_edit::ser::Error),
    #[error("a feature is unimplemented: {0}")]
    Unimplemented(String),
    #[error("a problem with utf-8 parsing occurred: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}
