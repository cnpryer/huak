use std::process::ExitCode;

use huak::Error;
use thiserror::Error as ThisError;

pub type CliResult<T> = Result<T, CliError>;
pub const BASIC_ERROR_CODE: ExitCode = ExitCode::FAILURE;

#[derive(Debug, ThisError)]
pub struct CliError {
    #[source]
    pub error: Error,
    pub exit_code: ExitCode,
    pub status_code: Option<i32>,
}

impl CliError {
    pub fn new(error: Error, exit_code: ExitCode) -> CliError {
        CliError {
            error,
            exit_code,
            status_code: None,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "huak exited with code {:?}: {}",
            self.exit_code, self.error
        )
    }
}

impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> CliError {
        CliError::new(Error::ClapError(err), BASIC_ERROR_CODE)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(Error::IOError(err), BASIC_ERROR_CODE)
    }
}

impl From<std::str::Utf8Error> for CliError {
    fn from(err: std::str::Utf8Error) -> CliError {
        CliError::new(Error::Utf8Error(err), BASIC_ERROR_CODE)
    }
}

impl From<std::env::VarError> for CliError {
    fn from(err: std::env::VarError) -> CliError {
        CliError::new(Error::EnvVarError(err), BASIC_ERROR_CODE)
    }
}
