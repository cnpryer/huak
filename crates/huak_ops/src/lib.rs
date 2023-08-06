//! # Huak
//!
//! A Python package manager written in Rust inspired by Cargo.
//!
//! ## About
//!
//! Huak is considered a package manager but focuses on supporting development workflows
//! useful for building both Python packages and projects in general.
//!
//! Workflows supported consist of the following life-cycle:
//! 1. Initialization and setup
//! 2. Making some change to the project
//! 3. Running tests
//! 4. Distributing the project
//!
//!```zsh
//! ❯ huak help
//!
//! A Python package manager written in Rust inspired by Cargo.
//!
//! Usage: huak [OPTIONS] <COMMAND>
//!
//! Commands:
//!   activate    Activate the virtual environment
//!   add         Add dependencies to the project
//!   build       Build tarball and wheel for the project
//!   completion  Generates a shell completion script for supported shells
//!   clean       Remove tarball and wheel from the built project
//!   fix         Auto-fix fixable lint conflicts
//!   fmt         Format the project's Python code
//!   init        Initialize the existing project
//!   install     Install the dependencies of an existing project
//!   lint        Lint the project's Python code
//!   new         Create a new project at <path>
//!   lish        Builds and uploads current project to a registry
//!   python      Manage Python installations
//!   remove      Remove dependencies from the project
//!   run         Run a command within the project's environment context
//!   test        Test the project's Python code
//!   update      Update the project's dependencies
//!   version     Display the version of the project
//!   help        Print this message or the help of the given subcommand(s)
//!
//!  Options:
//!    -q, --quiet    
//!    -h, --help     Print help
//!    -V, --version  Print version
//!```
mod config;
mod dependency;
mod environment;
mod error;
mod fs;
mod git;
mod metadata;
pub mod ops;
mod package;
mod python_environment;
mod sys;
mod version;
mod workspace;

pub use config::Config;
pub use error::{Error, HuakResult};
pub use fs::home_dir;
pub use python_environment::InstallOptions;
use python_environment::PythonEnvironment;
pub use sys::{SubprocessError, TerminalOptions, Verbosity};
pub use version::Version;
pub use workspace::{find_package_root, WorkspaceOptions};

#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
/// The resource directory found in the Huak repo used for testing purposes.
pub(crate) fn test_resources_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("dev-resources")
}
