use crate::error::HuakResult;
use crate::Error;
use std::{fmt::Display, io::Write, path::Path, process::Command};
use termcolor::{
    self, Color,
    Color::{Red, Yellow},
    ColorChoice, ColorSpec, StandardStream, WriteColor,
};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    #[default]
    Verbose,
    Normal,
    Quiet,
}

pub trait ToTerminal {
    /// Get a `Terminal`.
    fn to_terminal(&self) -> Terminal;
}

/// An abstraction around terminal output that remembers preferences for output
/// verbosity and color (inspired by cargo's Shell implementation).
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
            },
        }
    }

    /// Print an error message.
    pub fn print_error<T: Display>(&mut self, message: T) -> HuakResult<()> {
        self.output
            .message_stderr(&"error", Some(&message), Red, false)
    }

    /// Prints a warning message.
    pub fn print_warning<T: Display>(&mut self, message: T) -> HuakResult<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.print(&"warning", Some(&message), Yellow, false),
        }
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
        T: Display,
        U: Display,
    {
        self.print(&title, Some(&message), color, justified)
    }

    /// Prints a message, where the status will have `color` color, and can be justified.
    /// The messages follows without color.
    /// NOTE: Messages are printed to stderr. This is behavior cargo implements as well to
    /// avoid poluting stdout for end users. See https://github.com/rust-lang/cargo/issues/1473
    fn print(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
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

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
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

#[derive(Clone)]
pub struct TerminalOptions {
    pub verbosity: Verbosity,
}

impl TerminalOptions {
    pub fn verbosity(&self) -> &Verbosity {
        &self.verbosity
    }
}

pub fn parse_command_output(
    output: std::process::Output,
) -> HuakResult<String> {
    let mut s = String::new();
    s.push_str(std::str::from_utf8(&output.stdout)?);
    s.push_str(std::str::from_utf8(&output.stderr)?);
    Ok(s)
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
        #[allow(dead_code)]
        stdout: StandardStream,
        stderr: StandardStream,
    },
}

impl TerminalOut {
    /// Prints out a message with a status. The status comes first, and is bold plus
    /// the given color. The status can be justified, in which case the max width that
    /// will right align is DEFAULT_MESSAGE_JUSTIFIED_CHARS chars.
    fn message_stderr(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
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
}

/// Gets the name of the current shell.
pub fn shell_name() -> HuakResult<String> {
    let shell_path = shell_path()?;
    let shell_name = Path::new(&shell_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_owned())
        .ok_or_else(|| {
            Error::InternalError("shell path is invalid".to_owned())
        })?;
    Ok(shell_name)
}

/// Gets the path of the current shell from env var.
#[cfg(unix)]
pub fn shell_path() -> HuakResult<String> {
    std::env::var("SHELL").or(Ok("sh".to_string()))
}

/// Gets the path of the current shell from env var.
#[cfg(windows)]
pub fn shell_path() -> HuakResult<String> {
    Ok(std::env::var("COMSPEC")?)
}
