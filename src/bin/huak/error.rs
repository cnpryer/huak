use std::process::ExitCode;
use thiserror::Error as ThisError;

pub type CliResult<T> = Result<T, Error>;

#[derive(Debug, ThisError)]
pub struct Error {
    #[source]
    pub error: huak::Error,
    pub exit_code: ExitCode,
}

impl Error {
    pub fn new(error: huak::Error, exit_code: ExitCode) -> Error {
        Error { error, exit_code }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "huak exited with code {:?}: {}",
            self.exit_code, self.error
        )
    }
}

impl From<clap::Error> for Error {
    fn from(e: clap::Error) -> Error {
        Error::new(huak::Error::ClapError(e), ExitCode::FAILURE)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::new(huak::Error::IOError(e), ExitCode::FAILURE)
    }
}

impl From<std::io::ErrorKind> for Error {
    fn from(e: std::io::ErrorKind) -> Error {
        Error::new(huak::Error::InternalError(e.to_string()), ExitCode::FAILURE)
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Error {
        Error::new(huak::Error::EnvVarError(e), ExitCode::FAILURE)
    }
}
