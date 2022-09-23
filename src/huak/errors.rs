use std::fmt;

use anyhow::Error;

pub type CliResult<T> = std::result::Result<T, CliError>;

trait BinaryError {}

impl BinaryError for CliErrorType {}
impl BinaryError for Error {}

#[derive(Debug)]
pub enum CliErrorType {
    NotImplemented,
    MissingVirtualEnv,
    MissingArguments,
    UnknownError,
    IOError,
    UnknownCommand,
    DirectoryExists,
    AnyHowError(anyhow::Error),
    RuffError(String),
    PyBlackError(String),
    PyTest(String),
}

#[derive(Debug)]
pub struct CliError {
    pub error: CliErrorType,
}

impl CliError {
    pub fn new(mut error: CliErrorType) -> CliError {
        CliError { error }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This is a temporary value only useful for extracting something from anyhow::Error
        // It's something to do with the borrow checker as the value "does not live for long enough"
        // But I'm not knowledgeable enough to understand why.
        let binding: String;

        let error_string = match &self.error {
            CliErrorType::MissingArguments => "Some arguments were missing.",
            CliErrorType::IOError => "An IO error occurred.",
            CliErrorType::UnknownCommand => "This is an unknown command. Please check --help",
            CliErrorType::DirectoryExists => "This directory already exists/is not empty!",
            CliErrorType::AnyHowError(anyhow_error) => {
                binding = format!("AnyHow Error: {}", anyhow_error);
                binding.as_str()
            }
            CliErrorType::NotImplemented => "This is not implemented.",
            CliErrorType::MissingVirtualEnv => "This is missing a virtual environment.",
            CliErrorType::UnknownError => "An unknown error was raised. Please file a bug report",
            CliErrorType::RuffError(error) => error.as_str(),
            CliErrorType::PyBlackError(error) => error.as_str(),
            CliErrorType::PyTest(error) => error.as_str(),
            _ => "A strange unknown error was raised. Please file a bug report",
        };
        write!(f, "{}", error_string)
    }
}
impl From<anyhow::Error> for CliErrorType {
    fn from(err: anyhow::Error) -> CliErrorType {
        CliErrorType::AnyHowError(err)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> CliError {
        CliError::new(CliErrorType::AnyHowError(err))
    }
}

impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> CliError {
        let code = if err.use_stderr() { 1 } else { 0 };
        CliError::new(CliErrorType::AnyHowError(Error::from(err)))
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(CliErrorType::AnyHowError(Error::from(err)))
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
