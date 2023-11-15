use crate::error::HuakResult;
use crate::Error;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::{
    fmt::Display,
    io::Write,
    path::Path,
    process::{Command, ExitStatus},
};
use tempfile::TempDir;
use termcolor::{self, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub struct SubprocessError {
    status: ExitStatus,
}

impl SubprocessError {
    #[must_use]
    pub fn new(status: ExitStatus) -> Self {
        SubprocessError { status }
    }

    #[must_use]
    pub fn code(&self) -> Option<i32> {
        self.status.code()
    }
}

impl Display for SubprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.code())
    }
}

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
/// verbosity and color (inspired by cargo's `Shell`).
pub struct Terminal {
    /// A write object for terminal output.
    output: TerminalOut,
    /// How verbose messages should be.
    pub options: TerminalOptions,
}

impl Terminal {
    /// Create a new terminal struct with maximum verbosity.
    pub fn new() -> Terminal {
        Terminal {
            options: TerminalOptions {
                verbosity: Verbosity::Verbose,
                color_choice: ColorChoice::Auto,
            },
            output: TerminalOut::Stream {
                stderr: StandardStream::stderr(ColorChoice::Auto),
            },
        }
    }

    pub fn from_options(options: TerminalOptions) -> Terminal {
        let output = if options.color_choice == ColorChoice::Never {
            TerminalOut::Simple {
                stderr: StandardStream::stderr(ColorChoice::Never),
            }
        } else {
            TerminalOut::Stream {
                stderr: StandardStream::stderr(ColorChoice::Auto),
            }
        };

        Terminal { output, options }
    }

    /// Print an error message.
    pub fn print_error<T: Display>(&mut self, message: T) -> HuakResult<()> {
        self.output
            .message_stderr_with_status(&"error", Some(&message), Color::Red, false)
    }

    /// Prints a warning message.
    pub fn print_warning<T: Display>(&mut self, message: T) -> HuakResult<()> {
        match self.options.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.print(&"warning", Some(&message), Color::Yellow, false),
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

    /// Prints a message without a status.
    pub fn print_without_status<T>(&mut self, message: T, color: Color) -> HuakResult<()>
    where
        T: Display,
    {
        match self.options.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.output.message_stderr(Some(&message), color),
        }
    }

    /// Prints a message, where the status will have `color` color, and can be justified.
    /// The messages follows without color.
    ///
    /// NOTE: Messages are printed to stderr. This is behavior cargo implements as well to
    /// avoid polluting stdout for end users. See <https://github.com/rust-lang/cargo/issues/1473>.
    fn print(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
        color: Color,
        justified: bool,
    ) -> HuakResult<()> {
        match self.options.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self
                .output
                .message_stderr_with_status(status, message, color, justified),
        }
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.options.verbosity = verbosity;
    }

    /// Run a command from the terminal's context.
    pub fn run_command(&mut self, cmd: &mut Command) -> HuakResult<()> {
        // Allow `single_match_else` because `Quiet won't be the only handled `Verbosity`.
        #[allow(clippy::single_match_else)]
        let status = match self.options.verbosity {
            Verbosity::Quiet => {
                let output = cmd.output()?;
                let status = output.status;

                let stdout = trim_error_prefix(std::str::from_utf8(&output.stdout)?);
                let stderr = trim_error_prefix(std::str::from_utf8(&output.stderr)?);

                if !status.success() {
                    if !stdout.is_empty() {
                        self.print_error(stdout)?;
                    }
                    if !stderr.is_empty() {
                        self.print_error(stderr)?;
                    }
                }

                status
            }
            _ => {
                let mut child = cmd.spawn()?;

                match child.try_wait() {
                    Ok(Some(s)) => s,
                    Ok(None) => child.wait()?,
                    Err(e) => {
                        return Err(Error::from(e));
                    }
                }
            }
        };

        if !status.success() {
            return Err(Error::SubprocessFailure(SubprocessError::new(status)));
        }

        Ok(())
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct TerminalOptions {
    pub verbosity: Verbosity,
    pub color_choice: ColorChoice,
}

impl TerminalOptions {
    #[must_use]
    pub fn verbosity(&self) -> &Verbosity {
        &self.verbosity
    }

    #[must_use]
    pub fn color_choice(&self) -> &ColorChoice {
        &self.color_choice
    }

    #[must_use]
    pub fn take(self) -> TerminalOptions {
        self
    }
}

impl Default for TerminalOptions {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::default(),
            color_choice: ColorChoice::Auto,
        }
    }
}

pub fn parse_command_output(output: &std::process::Output) -> HuakResult<String> {
    let mut s = String::new();
    s.push_str(std::str::from_utf8(&output.stdout)?);
    s.push_str(std::str::from_utf8(&output.stderr)?);
    Ok(s)
}

fn trim_error_prefix(msg: &str) -> &str {
    msg.trim_start_matches("error:")
        .trim_start_matches("ERROR:")
        .trim_start()
}

/// Objects for writing terminal output to.
enum TerminalOut {
    Simple {
        stderr: StandardStream,
    },
    /// Color-enabled stdio with information on whether color should be used
    Stream {
        stderr: StandardStream,
    },
}

impl TerminalOut {
    /// Prints out a message with a status. The status comes first, and is bold plus
    /// the given color. The status can be justified, in which case the max width that
    /// will right align is `DEFAULT_MESSAGE_JUSTIFIED_CHARS` chars.
    fn message_stderr_with_status(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
        color: termcolor::Color,
        justified: bool,
    ) -> HuakResult<()> {
        match *self {
            TerminalOut::Stream { ref mut stderr, .. } => {
                stderr.reset()?;
                stderr.set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))?;
                if justified {
                    write!(stderr, "  {status:>10}")?;
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
            TerminalOut::Simple { ref mut stderr, .. } => {
                write!(stderr, "{status}")?;
                write!(stderr, ":")?;

                match message {
                    Some(message) => writeln!(stderr, " {message}")?,
                    None => write!(stderr, " ")?,
                }
            }
        }
        Ok(())
    }

    fn message_stderr(
        &mut self,
        message: Option<&dyn Display>,
        color: termcolor::Color,
    ) -> HuakResult<()> {
        match *self {
            TerminalOut::Stream { ref mut stderr, .. } => {
                stderr.reset()?;
                stderr.set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))?;
                match message {
                    Some(message) => writeln!(stderr, "{message}")?,
                    None => write!(stderr, " ")?,
                }
            }
            TerminalOut::Simple { ref mut stderr, .. } => match message {
                Some(message) => writeln!(stderr, "{message}")?,
                None => write!(stderr, " ")?,
            },
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
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::InternalError("shell path is invalid".to_owned()))?;
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

pub(crate) fn symlink_supported() -> bool {
    if cfg!(unix) {
        true
    } else {
        test_symlink().is_ok()
    }
}

fn test_symlink() -> HuakResult<()> {
    let dir = TempDir::new()?;
    let original = dir.path().join("file");
    std::fs::write(&original, "")?;
    try_symlink(original, dir.path().join("link"))
}

// TODO(cnpryer): Refactor (see huak-toolchain)
fn try_symlink<T: AsRef<Path>>(original: T, link: T) -> Result<(), Error> {
    #[cfg(unix)]
    let err = symlink(original, link);

    #[cfg(windows)]
    let err = symlink_file(original, link);

    Ok(err?)
}
