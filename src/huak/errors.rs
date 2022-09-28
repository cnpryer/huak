use std::{fmt, process::ExitCode};

use anyhow::Error;

pub type CliResult<T> = std::result::Result<T, CliError>;

trait BinaryError {}

impl BinaryError for HuakError {}
impl BinaryError for Error {}

const BASIC_ERROR_CODE: ExitCode = ExitCode::FAILURE;

// TODO: Slit into different types of errors. This could be
//       based on behavior, data, tooling, etc.
#[derive(Debug)]
pub enum HuakError {
    NotImplemented,
    MissingArguments,
    UnknownError,
    IOError,
    UnknownCommand,
    DirectoryExists,
    AnyHowError(anyhow::Error),
    // TODO: Abstract out wrapped cli errors.
    RuffError(Box<CliError>),
    PyBlackError(Box<CliError>),
    PyTestError(Box<CliError>),
    PythonNotFound,
    VenvNotFound,
    PyProjectTomlNotFound, // TODO: Manfiest
    PackageInstallFailure(String),
}

#[derive(Debug)]
pub struct CliError {
    pub error: HuakError,
    pub exit_code: ExitCode,
    pub status_code: Option<i32>,
}

impl CliError {
    pub fn new(error: HuakError, exit_code: ExitCode) -> CliError {
        CliError {
            error,
            exit_code,
            status_code: None,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This is a temporary value only useful for extracting something from anyhow::Error
        // It's something to do with the borrow checker as the value "does not live for long enough"
        // But I'm not knowledgeable enough to understand why.
        let binding: String;

        let error_string = match &self.error {
            HuakError::MissingArguments => "Some arguments were missing.",
            HuakError::IOError => "An IO error occurred.",
            HuakError::UnknownCommand => {
                "This is an unknown command. Please check --help."
            }
            HuakError::DirectoryExists => {
                "This directory already exists and may not be empty!"
            }
            HuakError::AnyHowError(anyhow_error) => {
                binding = format!("An error occurred: {}", anyhow_error);
                binding.as_str()
            }
            HuakError::NotImplemented => {
                "This feature is not implemented. \
                See https://github.com/cnpryer/huak/milestones."
            }
            HuakError::VenvNotFound => "No venv was found.",
            HuakError::UnknownError => {
                "An unknown error occurred. Please file a bug report here \
                https://github.com/cnpryer/huak/issues/new?\
                assignees=&labels=bug&template=BUG_REPORT.md&title="
            }
            HuakError::RuffError(err) => {
                binding = format!("Ruff Error: {err}");
                binding.as_str()
            }
            HuakError::PyBlackError(err) => {
                binding = format!("Black Error: {err}");
                binding.as_str()
            }
            HuakError::PyTestError(err) => {
                binding = format!("Pytest Error: {err}");
                binding.as_str()
            }
            HuakError::PythonNotFound => {
                "Python was not found on your operating system. \
                Please install Python at https://www.python.org/."
            }
            HuakError::PyProjectTomlNotFound => {
                "A pyproject.toml could not be found."
            }
            HuakError::PackageInstallFailure(package) => {
                binding = format!("Failed to install package: {package}.");
                binding.as_str()
            }
        };
        write!(f, "{}", error_string)
    }
}
impl From<anyhow::Error> for HuakError {
    fn from(err: anyhow::Error) -> HuakError {
        HuakError::AnyHowError(err)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> CliError {
        CliError::new(HuakError::AnyHowError(err), BASIC_ERROR_CODE)
    }
}

impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> CliError {
        CliError::new(
            HuakError::AnyHowError(Error::from(err)),
            BASIC_ERROR_CODE,
        )
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(
            HuakError::AnyHowError(Error::from(err)),
            BASIC_ERROR_CODE,
        )
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> CliError {
        CliError::new(
            HuakError::AnyHowError(Error::from(err)),
            BASIC_ERROR_CODE,
        )
    }
}

pub fn internal<S: fmt::Display>(error: S) -> anyhow::Error {
    InternalError::new(anyhow::format_err!("{}", error)).into()
}

/// An unexpected, internal error.
///
/// This should only be used for unexpected errors. It prints a message asking
/// the user to file a bug report.
pub struct InternalError {
    inner: Error,
}

impl InternalError {
    pub fn new(inner: Error) -> InternalError {
        InternalError { inner }
    }
}

impl std::error::Error for InternalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
