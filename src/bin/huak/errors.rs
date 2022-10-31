use std::process::ExitCode;

use huak::errors::HuakError;
use thiserror::Error;

pub type CliResult<T> = Result<T, CliError>;
pub const BASIC_ERROR_CODE: ExitCode = ExitCode::FAILURE;

#[derive(Debug, Error)]
pub struct CliError {
    #[source]
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
        CliError::new(HuakError::ClapError(err), BASIC_ERROR_CODE)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(HuakError::IOError(err), BASIC_ERROR_CODE)
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> CliError {
        CliError::new(HuakError::HttpError(err), BASIC_ERROR_CODE)
    }
}

impl From<std::str::Utf8Error> for CliError {
    fn from(err: std::str::Utf8Error) -> CliError {
        CliError::new(HuakError::Utf8Error(err), BASIC_ERROR_CODE)
    }
}

impl From<std::env::VarError> for CliError {
    fn from(err: std::env::VarError) -> CliError {
        CliError::new(HuakError::EnvVarError(err), BASIC_ERROR_CODE)
    }
}
