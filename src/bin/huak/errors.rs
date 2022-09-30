use thiserror::Error;

pub type CliResult<T> = Result<T, CliError>;

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

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "hauk exited with code {:?}: {}",
            self.exit_code, self.error
        )
    }
}

// impl fmt::Display for CliError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // This is a temporary value only useful for extracting something from anyhow::Error
//         // It's something to do with the borrow checker as the value "does not live for long enough"
//         // But I'm not knowledgeable enough to understand why.
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
//             // HuakError::AnyHowError(anyhow_error) => {
//             //     binding = format!("An error occurred: {}", anyhow_error);
//             //     binding.as_str()
//             // }
//             HuakError::NotImplemented => {
//                 "This feature is not implemented. \
//                 See https://github.com/cnpryer/huak/milestones."
//             }
//             HuakError::VenvNotFound => "No venv was found.",
//             HuakError::UnknownError => {
// "An unknown error occurred. Please file a bug report here \
// https://github.com/cnpryer/huak/issues/new?\
// assignees=&labels=bug&template=BUG_REPORT.md&title="
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
//             HuakError::PackageInstallFailure(package) => {
//                 binding = format!("Failed to install package: {package}.");
//                 binding.as_str()
//             }
//         };
//         write!(f, "{}", error_string)
//     }
// }

// impl From<anyhow::Error> for HuakError {
//     fn from(err: anyhow::Error) -> HuakError {
//         HuakError::AnyHowError(err)
//     }
// }

// impl From<anyhow::Error> for CliError {
//     fn from(err: anyhow::Error) -> CliError {
//         CliError::new(HuakError::AnyHowError(err), BASIC_ERROR_CODE)
//     }
// }

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

pub fn internal<S: fmt::Display>(error: S) -> HuakError {
    HuakError::UnknownError(error.to_string())
}

// pub fn internal<S: fmt::Display>(error: S) -> anyhow::Error {
//     InternalError::new(anyhow::format_err!("{}", error)).into()
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
