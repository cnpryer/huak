use std::{io, process::ExitCode};

// use anyhow::Error;
use thiserror::Error;

trait BinaryError {}

impl BinaryError for HuakError {}
// impl BinaryError for Error {}

const BASIC_ERROR_CODE: ExitCode = ExitCode::FAILURE;

pub type HuakResult<T> = Result<T, HuakError>;

// TODO: Slit into different types of errors. This could be
//       based on behavior, data, tooling, etc.
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
    DirectoryExists(String),
    #[error("An HTTP error occurred: {0}.")]
    HttpError(#[from] reqwest::Error),
    #[error("An error occurred while parsing bytes to a string: {0}.")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    InternalError(String),
    // AnyHowError(anyhow::Error),
    // TODO: Abstract out wrapped cli errors.
    // FIXME: This presents a circular problem. CliError contains a HuakError, so there should
    // not be any enum variants containing a CliError. Instead, isolate CliError to only in the
    // `bin` crate. Add some sort of `CommandError` to `HuakError`, and use that instead.
    // I see you encountered this and boxed it, but down the road I feel this just might make
    // things more complicated.
    // #[error("Ruff Error: {0}")]
    // RuffError(Box<CliError>),
    // #[error("Black Error: {0}")]
    // PyBlackError(Box<CliError>),
    // #[error("Pytest Error: {0}")]
    // PyTestError(Box<CliError>),
    #[error(
        "Python was not found on your operating system. Please \
        install Python at https://www.python.org/."
    )]
    PythonNotFound,
    #[error("No venv was found.")]
    VenvNotFound,
    #[error("A pyproject.toml could not be found.")]
    PyProjectTomlNotFound, // TODO: Manfiest
    #[error("Failed to install Python package: {0}.")]
    PyPackageInstallFailure(String),
    // TODO: had some rebase conflicts, leaving this for now but seems like duplicate...
    #[error("Failed to install package: {0}.")]
    PackageInstallFailure(String),
    #[error("Failed to init Python package: {0}.")]
    PyPackageInitError(String),
    #[error("Invalid Python package version operator: {0}.")]
    InvalidPyPackageVersionOp(String),
    #[error("Failed to build the project.")]
    BuildFailure,
}

// impl fmt::Display for CliError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let binding: String;

//         let error_string = match &self.error {
//             HuakError::MissingArguments => "Some arguments were missing.",
//             HuakError::IOError => "An IO error occurred.",
//             HuakError::UnknownCommand => {
//                 "This is an unknown command. Please check --help."
//             }
//             HuakError::DirectoryExists => {
//                 "This directory already exists and may not be empty!"
//             }
//             HuakError::AnyHowError(anyhow_error) => {
//                 binding = format!("An error occurred: {}", anyhow_error);
//                 binding.as_str()
//             }
//             HuakError::NotImplemented => {
//                 "This feature is not implemented. \
//                 See https://github.com/cnpryer/huak/milestones."
//             }
//             HuakError::VenvNotFound => "No venv was found.",
//             HuakError::UnknownError => {
//                 "An unknown error occurred. Please file a bug report here \
//                 https://github.com/cnpryer/huak/issues/new?\
//                 assignees=&labels=bug&template=BUG_REPORT.md&title="
//             }
//             HuakError::RuffError(err) => {
//                 binding = format!("Ruff Error: {err}");
//                 binding.as_str()
//             }
//             HuakError::PyBlackError(err) => {
//                 binding = format!("Black Error: {err}");
//                 binding.as_str()
//             }
//             HuakError::PyTestError(err) => {
//                 binding = format!("Pytest Error: {err}");
//                 binding.as_str()
//             }
//             HuakError::PythonNotFound => {
//                 "Python was not found on your operating system. \
//                 Please install Python at https://www.python.org/."
//             }
//             HuakError::PyProjectTomlNotFound => {
//                 "A pyproject.toml could not be found."
//             }
//             HuakError::PyPackageInstallFailure(package) => {
//                 binding =
//                     format!("Failed to install Python package: {package}.");
//                 binding.as_str()
//             }
//             HuakError::PyPackageInitError(package) => {
//                 binding = format!("Failed to init Python package: {package}.");
//                 binding.as_str()
//             }
//             HuakError::InvalidPyPackageVersionOp(op) => {
//                 binding =
//                     format!("Invalid Python package version operator: {op}.");
//                 binding.as_str()
//             }
//             HuakError::BuildFailure => "Failed to build the project.",
//         };
//         write!(f, "{}", error_string)
//     }
// }