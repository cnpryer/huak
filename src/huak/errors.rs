use std::fmt;

use anyhow::Error;
use crate::errors::CliErrorCode::{AnyHowError, UnknownError};
pub type CliResult<T> = std::result::Result<T, CliError>;

trait BinaryError{}

impl BinaryError for CliErrorCode {}
impl BinaryError for Error {}

#[derive(Debug)]
pub enum CliErrorCode {
    NotImplemented,
    MissingVirtualEnv,
    MissingArguments,
    UnknownError,
    IOError,
    UnknownCommand,
    DirectoryExists,
    AnyHowError(Error)
}

#[derive(Debug)]
pub struct CliError {
    pub error: CliErrorCode
}

impl CliError {
    pub fn new(mut error_code: CliErrorCode) -> CliError {
        CliError {
            error: error_code
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_string = match &self.error {
            CliErrorCode::MissingArguments => "Some arguments were missing.",
            CliErrorCode::IOError => "An IO error occured.",
            CliErrorCode::UnknownCommand => "This is an unknown command. Please check --help",
            CliErrorCode::DirectoryExists => "This directory already exists/is not empty!",
            CliErrorCode::AnyHowError(error ) => "An AnyHow error occured",
            CliErrorCode::NotImplemented => "This is not implemented.",
            CliErrorCode::MissingVirtualEnv => "This is missing a virtual environment.",
            CliErrorCode::UnknownError => "An unknown error was raised. Please file a bug report",
            _ => "A strange unknown error was raised. Please file a bug report"
        };
        write!(f, "{}", error_string)
    }
}
impl From<anyhow::Error> for CliErrorCode {
    fn from(err: anyhow::Error) -> CliErrorCode {
        CliErrorCode::AnyHowError(err)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> CliError {
        CliError::new(CliErrorCode::AnyHowError(err))
    }
}

impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> CliError {
        let code = if err.use_stderr() { 1 } else { 0 };
        CliError::new(AnyHowError(Error::from(err)))
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(AnyHowError(Error::from(err)))
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