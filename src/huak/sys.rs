use crate::error::HuakResult;
use crate::Error;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{ffi::OsString, path::PathBuf};
use termcolor::{self, Color, ColorSpec, StandardStream, WriteColor};
use termcolor::{
    Color::{Cyan, Green, Red, Yellow},
    ColorChoice,
};

/// Get a vector of paths from the system PATH environment variable.
pub fn env_path_values() -> Vec<PathBuf> {
    std::env::split_paths(&env_path_string()).collect()
}

pub fn env_path_string() -> OsString {
    match std::env::var_os("PATH") {
        Some(val) => val,
        None => OsString::new(),
    }
}

/// Get the VIRTUAL_ENV environment path if it exists.
pub fn active_virtual_env_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("VIRTUAL_ENV") {
        return Some(PathBuf::from(path));
    }
    None
}

/// Get the CONDA_PREFIX environment path if it exists.
pub fn active_conda_env_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CONDA_PREFIX") {
        return Some(PathBuf::from(path));
    }
    None
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    #[default]
    Verbose,
    Normal,
    Quiet,
}

/// An abstraction around terminal output that remembers preferences for output
/// verbosity and color (inspired by cargo's implementation).
pub struct Terminal {
    /// A write object for terminal output.
    output: TerminalOut,
    /// How verbose messages should be.
    verbosity: Verbosity,
}

impl Terminal {
    /// Create a new terminal struct with maximum verbosity.
    pub fn new() -> Terminal {
        Terminal {
            verbosity: Verbosity::Verbose,
            output: TerminalOut::Stream {
                stdout: StandardStream::stdout(ColorChoice::Auto),
                stderr: StandardStream::stderr(ColorChoice::Auto),
                color_choice: ColorChoice::Auto,
            },
        }
    }

    /// Shortcut to right-align and color green a status message.
    pub fn status<T, U>(&mut self, status: T, message: U) -> HuakResult<()>
    where
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
        self.print(&status, Some(&message), Green, true)
    }

    pub fn status_header<T>(&mut self, status: T) -> HuakResult<()>
    where
        T: std::fmt::Display,
    {
        self.print(&status, None, Cyan, true)
    }

    /// Shortcut to right-align a status message.
    pub fn status_with_color<T, U>(
        &mut self,
        status: T,
        message: U,
        color: Color,
    ) -> HuakResult<()>
    where
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
        self.print(&status, Some(&message), color, true)
    }

    /// Print an error message.
    pub fn print_error<T: std::fmt::Display>(
        &mut self,
        message: T,
    ) -> HuakResult<()> {
        self.output
            .message_stderr(&"error", Some(&message), Red, false)
    }

    /// Prints a warning message.
    pub fn print_warning<T: std::fmt::Display>(
        &mut self,
        message: T,
    ) -> HuakResult<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.print(&"warning", Some(&message), Yellow, false),
        }
    }

    /// Prints a note message.
    pub fn print_note<T: std::fmt::Display>(
        &mut self,
        message: T,
    ) -> HuakResult<()> {
        self.print(&"note", Some(&message), Cyan, false)
    }

    /// Prints a custom message.
    pub fn print_custom<T, U>(
        &mut self,
        title: U,
        message: T,
        color: Color,
        justified: bool,
    ) -> HuakResult<()>
    where
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
        self.print(&title, Some(&message), color, justified)
    }

    /// Prints a message, where the status will have `color` color, and can be justified.
    /// The messages follows without color.
    /// NOTE: Messages are printed to stderr. This is behavior cargo implements as well to
    /// avoid poluting stdout for end users. See https://github.com/rust-lang/cargo/issues/1473
    fn print(
        &mut self,
        status: &dyn std::fmt::Display,
        message: Option<&dyn std::fmt::Display>,
        color: Color,
        justified: bool,
    ) -> HuakResult<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self
                .output
                .message_stderr(status, message, color, justified),
        }
    }

    /// Gets a reference to the underlying stdout writer.
    pub fn stdout(&mut self) -> &mut dyn Write {
        self.output.stdout()
    }

    /// Gets a reference to the underlying stderr writer.
    pub fn stderr(&mut self) -> &mut dyn Write {
        self.output.stderr()
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    /// Get a reference to the verbosity level.
    pub fn verbosity(&self) -> &Verbosity {
        &self.verbosity
    }

    /// Gets the current color choice.
    ///
    /// If we are not using a color stream, this will always return `Never`, even if the color
    /// choice has been set to something else.
    pub fn color_choice(&self) -> ColorChoice {
        match self.output {
            TerminalOut::Stream { color_choice, .. } => color_choice,
        }
    }

    /// Run a command from the terminal's context.
    pub fn run_command(&mut self, cmd: &mut Command) -> HuakResult<()> {
        let code = match self.verbosity {
            Verbosity::Quiet => {
                let output = cmd.output()?;
                let status = output.status;
                let stdout =
                    trim_error_prefix(std::str::from_utf8(&output.stdout)?);
                let stderr =
                    trim_error_prefix(std::str::from_utf8(&output.stderr)?);
                let code = status.code().unwrap_or_default();
                if code > 0 {
                    if !stdout.is_empty() {
                        self.print_error(stdout)?;
                    }
                    if !stderr.is_empty() {
                        self.print_error(stderr)?;
                    }
                }
                code
            }
            _ => {
                let mut child = cmd.spawn()?;
                let status = match child.try_wait() {
                    Ok(Some(s)) => s,
                    Ok(None) => child.wait()?,
                    Err(e) => {
                        return Err(Error::from(e));
                    }
                };
                status.code().unwrap_or_default()
            }
        };
        if code > 0 {
            std::process::exit(code)
        }
        Ok(())
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

fn trim_error_prefix(msg: &str) -> &str {
    msg.trim_start_matches("error:")
        .trim_start_matches("ERROR:")
        .trim_start()
}

/// Objects for writing terminal output to.
enum TerminalOut {
    /// Color-enabled stdio with information on whether color should be used
    Stream {
        stdout: StandardStream,
        stderr: StandardStream,
        color_choice: ColorChoice,
    },
}

impl TerminalOut {
    /// Prints out a message with a status. The status comes first, and is bold plus
    /// the given color. The status can be justified, in which case the max width that
    /// will right align is DEFAULT_MESSAGE_JUSTIFIED_CHARS chars.
    fn message_stderr(
        &mut self,
        status: &dyn std::fmt::Display,
        message: Option<&dyn std::fmt::Display>,
        color: Color,
        justified: bool,
    ) -> HuakResult<()> {
        match *self {
            TerminalOut::Stream { ref mut stderr, .. } => {
                stderr.reset()?;
                stderr.set_color(
                    ColorSpec::new().set_bold(true).set_fg(Some(color)),
                )?;
                if justified {
                    write!(stderr, "{status:>12}")?;
                } else {
                    write!(stderr, "{status}")?;
                    stderr.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stderr, ":")?;
                }
                stderr.reset()?;
                match message {
                    Some(message) => writeln!(stderr, " {message}")?,
                    None => write!(stderr, " ")?,
                }
            }
        }
        Ok(())
    }

    /// Get a mutable reference to the stdout writer.
    pub fn stdout(&mut self) -> &mut dyn Write {
        match *self {
            TerminalOut::Stream { ref mut stdout, .. } => stdout,
        }
    }

    /// Get a mutable reference to the stderr writer.
    pub fn stderr(&mut self) -> &mut dyn Write {
        match *self {
            TerminalOut::Stream { ref mut stderr, .. } => stderr,
        }
    }
}

#[derive(Default)]
pub struct TerminalOptions {
    pub verbosity: Verbosity,
}

/// Gets the name of the current shell.
///
/// Returns an error if it fails to get correct env vars.
pub fn shell_name() -> HuakResult<String> {
    let shell_path = shell_path()?;
    let shell_name = Path::new(&shell_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_owned())
        .ok_or_else(|| {
            Error::InternalError("shell path is invalid".to_owned())
        });

    shell_name
}

/// Gets the path of the current shell from env var
///
/// Returns an error if it fails to get correct env vars.
#[cfg(unix)]
pub fn shell_path() -> HuakResult<String> {
    std::env::var("SHELL").or(Ok("sh".to_string()))
}

/// Gets the path of the current shell from env var
///
/// Returns an error if it fails to get correct env vars.
#[cfg(windows)]
pub fn shell_path() -> HuakResult<String> {
    Ok(std::env::var("COMSPEC")?)
}
