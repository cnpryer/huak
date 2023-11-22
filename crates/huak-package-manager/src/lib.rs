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

mod config;
mod dependency;
mod environment;
mod error;
mod fs;
mod git;
mod manifest;
pub mod ops;
mod package;
mod python_environment;
mod sys;
mod workspace;

pub use config::Config;
pub use dependency::{dependency_iter, Dependency};
pub use environment::{env_path_string, env_path_values, Environment};
pub use error::{Error, HuakResult};
pub use fs::{copy_dir, last_path_component, CopyDirOptions};
pub use git::{default_python_gitignore, init as git_init};
pub use manifest::{
    default_package_entrypoint_string, default_package_test_file_contents,
    default_pyproject_toml_contents, LocalManifest,
};
pub use package::{importable_package_name, Package};
pub use python_environment::{
    active_python_env_path, directory_is_venv, initialize_venv, venv_executables_dir_path,
    InstallOptions, PythonEnvironment,
};
pub use sys::{shell_name, shell_path, SubprocessError, TerminalOptions, Verbosity};
pub use workspace::{Workspace, WorkspaceOptions};
