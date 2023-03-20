use crate::error::HuakResult;
use pep440_rs::Version;
use std::io::Write;
use std::process::Command;
use std::{collections::HashMap, ffi::OsString, path::PathBuf};
use termcolor::{self, Color, ColorSpec, StandardStream, WriteColor};
use termcolor::{
    Color::{Cyan, Green, Red, Yellow},
    ColorChoice,
};

/// A struct to contain useful platform data and objects.
pub struct Platform {
    /// The name of the platform.
    name: String,
    /// Absolute paths to each Python interpreter installed.
    python_paths: HashMap<Version, PathBuf>,
    /// An abstraction for the terminal.
    terminal: Terminal,
}

impl Platform {
    /// Create a new platform.
    pub fn new() -> Platform {
        todo!()
    }

    /// Install a Python interpreter.
    pub fn install_python(&mut self, version_str: &str) -> HuakResult<()> {
        todo!()
    }

    /// Get the absolute path to a specific Python interpreter with a version &str.
    pub fn python_path(&self, version_str: &str) -> Option<&PathBuf> {
        todo!()
    }

    /// Get the absolute path to the latest version Python interpreter installed.
    pub fn python_path_latest(&self) -> Option<&PathBuf> {
        todo!()
    }

    /// Get a reference to the platform's terminal.
    pub fn terminal(&self) -> &Terminal {
        &self.terminal
    }
}

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
            TerminalOut::Write(_) => ColorChoice::Never,
        }
    }

    /// Run a command from the terminal's context.
    pub fn run_command(&mut self, cmd: &mut Command) -> HuakResult<()> {
        todo!()
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

/// Objects for writing terminal output to.
enum TerminalOut {
    /// A basic write object without support for color
    Write(Box<dyn Write>),
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
            TerminalOut::Write(ref mut w) => {
                if justified {
                    write!(w, "{:>12}", status)?;
                } else {
                    write!(w, "{}:", status)?;
                }
                match message {
                    Some(message) => writeln!(w, " {}", message)?,
                    None => write!(w, " ")?,
                }
            }
            TerminalOut::Stream { ref mut stderr, .. } => {
                stderr.reset()?;
                stderr.set_color(
                    ColorSpec::new().set_bold(true).set_fg(Some(color)),
                )?;
                if justified {
                    write!(stderr, "{:>12}", status)?;
                } else {
                    write!(stderr, "{}", status)?;
                    stderr.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stderr, ":")?;
                }
                stderr.reset()?;
                match message {
                    Some(message) => writeln!(stderr, " {}", message)?,
                    None => write!(stderr, " ")?,
                }
            }
        }
        Ok(())
    }

    /// Get a mutable reference to the stdout writer.
    pub fn stdout(&mut self) -> &mut dyn Write {
        match *self {
            TerminalOut::Write(ref mut w) => w,
            TerminalOut::Stream { ref mut stdout, .. } => stdout,
        }
    }

    /// Get a mutable reference to the stderr writer.
    pub fn stderr(&mut self) -> &mut dyn Write {
        match *self {
            TerminalOut::Write(ref mut w) => w,
            TerminalOut::Stream { ref mut stderr, .. } => stderr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_terminal_command() {
        let platform = Platform::new();

        todo!()
    }
}
