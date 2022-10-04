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

// TODO: not sure how you'd like to go about determining exit/status codes, but
// you can do some sort of impl From<HuakError> for CliError, or maybe
// impl From<HuakError> for ExitCode? Can also just do a function if you wanted

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "hauk exited with code {:?}: {}",
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

// pub fn internal<S: std::fmt::Display>(error: S) -> HuakError {
//     HuakError::UnknownError(error.to_string())
// }
