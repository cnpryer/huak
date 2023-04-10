use std::path::PathBuf;

use crate::{sys::Terminal, workspace::Workspace, TerminalOptions};

#[derive(Clone)]
/// The main `Config` for Huak.
///
/// The `Config` contains data telling Huak what to do during at times.
/// An example would be indicating what the initial `Workspace` root should be or
/// what the current working directory was at the time an operation was requested.
///
/// ```
/// use huak::{Config, sys::{TerminalOptions, Verbosity};
///
/// let config = Config {
///     workspace_root: PathBuf::from("."),
///     cwd: PathBuf::from("."),
///     terminal_options: TerminalOptions {
///         verbosity: Verbosity::Normal,
///     }
/// };
///
/// let workspace = config.workspace();
/// ```
pub struct Config {
    /// The configured `Workspace` root path.
    pub workspace_root: PathBuf,
    /// The current working directory where Huak was invoked or otherwise requested from.
    pub cwd: PathBuf,
    /// `Terminal` options to use.
    pub terminal_options: TerminalOptions,
}

impl Config {
    /// Resolve the current workspace based on the `Config` data.
    pub fn workspace(&self) -> Workspace {
        Workspace::new(&self.workspace_root, self)
    }

    /// Get a `Terminal` based on the `Config` data.
    pub fn terminal(&self) -> Terminal {
        let mut terminal = Terminal::new();
        let verbosity = *self.terminal_options.verbosity();
        terminal.set_verbosity(verbosity);

        terminal
    }
}
