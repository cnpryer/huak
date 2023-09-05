use std::path::PathBuf;

use crate::{sys::Terminal, workspace::Workspace, TerminalOptions};

#[derive(Clone, Default)]
/// The main `Config` for Huak.
///
/// The `Config` contains data telling Huak what to do at times.
/// An example would be indicating what the initial `Workspace` root should be, or
/// what it was when it was requested.
///
/// ```
/// use huak_ops::{Config, sys::{TerminalOptions, Verbosity};
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
    /// The current working directory.
    pub cwd: PathBuf,
    /// `Terminal` options to use.
    pub terminal_options: TerminalOptions,
}

impl Config {
    /// Resolve the current `Workspace` based on the `Config` data.
    pub fn workspace(&self) -> Workspace {
        Workspace::new(&self.workspace_root, self)
    }

    /// Get a `Terminal` based on the `Config` data.
    pub fn terminal(&self) -> Terminal {
        Terminal::from_options(self.terminal_options.clone())
    }

    /// Get a `Config` with a new `Terminal`.
    pub fn with_terminal(self, terminal_options: TerminalOptions) -> Config {
        Config {
            workspace_root: self.workspace_root,
            cwd: self.cwd,
            terminal_options,
        }
    }
}
