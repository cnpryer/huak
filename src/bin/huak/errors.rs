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
// impl From<HuakError> for ExitCode?

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

// Note: I have commented this out temporarily. I see you already had an `UnknownError` enum
// for `HuakError`. I think this is a good idea, wasn't sure InternalError was necessary as well.
// / An unexpected, internal error.
// /
// / This should only be used for unexpected errors. It prints a message asking
// / the user to file a bug report.
// pub struct InternalError {
//     inner: Error,
// }

// impl InternalError {
//     pub fn new(inner: Error) -> InternalError {
//         InternalError { inner }
//     }
// }

// impl std::error::Error for InternalError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         self.inner.source()
//     }
// }

// impl fmt::Debug for InternalError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.inner.fmt(f)
//     }
// }

// impl fmt::Display for InternalError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.inner.fmt(f)
//     }
// }
