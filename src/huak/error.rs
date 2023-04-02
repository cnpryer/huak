use std::{io, path::PathBuf};

use thiserror::Error as ThisError;

pub type HuakResult<T> = Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("a problem occurred with resolving build options")]
    BuildOptionsMissingError,
    #[error("a problem with argument parsing occurred: {0}")]
    ClapError(#[from] clap::Error),
    #[error("a problem with dependency resolution occurred: {0}")]
    DependencyResolutionError(String),
    #[error("a directory already exists: {0}")]
    DirectoryExists(PathBuf),
    #[error("a problem with the environment occurred: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("a problem with the pseudo-terminal occurred: {0}")]
    FormatterError(String),
    #[error("a problem occurred with resolving format options")]
    FormatOptionsMissingError,
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
    #[error("a problem with the linter occurred: {0}")]
    LinterError(String),
    #[error("a problem occurred with resolving lint options")]
    LintOptionsMissingError,
    #[error("a problem with building the project occurred")]
    PackageBuildError,
    #[error("a problem occurred initializing a dependency from a string")]
    DependencyFromStringError(String),
    #[error("a problem with the package index occurred: {0}")]
    PackageIndexError(String),
    #[error("a problem with package installation occurred: {0}")]
    PackageInstallationError(String),
    #[error("a problem with the package version operator occurred: {0}")]
    PackageInvalidVersionOperator(String),
    #[error("a problem with the package version occurred: {0}")]
    PackageInvalidVersion(String),
    #[error("a problem with the package version specifier occurred")]
    PackageVersionSpecifierError,
    #[error("a project already exists")]
    ProjectExistsError,
    #[error("a manifest file could not be found")]
    ProjectManifestNotFoundError,
    #[error("a manifest file already exists")]
    ProjectManifestExistsError,
    #[error("a project could not be found")]
    ProjectNotFoundError,
    #[error("a problem with locating the project's version number occurred")]
    ProjectVersionNotFound,
    #[error("a problem occurred attempting to locate the project's root")]
    ProjectRootMissingError,
    #[error("a problem occurred with resolving publish options")]
    PublishOptionsMissingError,
    #[error("an installed python module could not be found: {0}")]
    PythonModuleMissingError(String),
    #[error("a python interpreter could not be found")]
    PythonNotFoundError,
    #[error("a feature is unimplemented: {0}")]
    UnimplementedError(String),
    #[error(
        "a problem occurred parsing the virtual environment's config file: {0}"
    )]
    VenvInvalidConfigFile(String),
    #[error("a python environment could not be found")]
    PythonEnvironmentNotFoundError,
    #[error("a regex error occurred: {0}")]
    RegexError(#[from] regex::Error),
    #[error("a http request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("a problem with the test utility occurred: {0}")]
    TestingError(String),
    #[error("a problem with toml deserialization occurred: {0}")]
    TOMLDeserializationError(#[from] toml::de::Error),
    #[error("a problem with toml serialization occurred {0}")]
    TOMLSerializationError(#[from] toml::ser::Error),
    #[error("a problem with toml deserialization occurred: {0}")]
    TOMLEditDeserializationError(#[from] toml_edit::de::Error),
    #[error("a problem with toml serialization occurred {0}")]
    TOMLEditSerializationError(#[from] toml_edit::ser::Error),
    #[error("a problem with utf-8 parsing occurred: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    CommandError(String),
}
