//! # Python Manager
//!
//!  A Python interpreter management system.
//!
//! NOTE: This crate is a work-in-progress.
//!
//! ## Usage
//!
//! ### CLI
//!
//! ```cmd
//! huak_python_manager install 3.12 --target <path-to-target-dir>
//! ```
//!
//! ### Rust
//!
//! #### Install a Python release to a target directory with minimal configuration.
//!
//! TODO(cnpryer): Design better API.
//!
//! ```no_run
//! use std::{str::FromStr, path::PathBuf};
//! use huak_python_manager::{Options, RequestedVersion, Strategy, install_with_target, resolve_release};
//!
//!
//! // The version of the Python to install.
//! let version = RequestedVersion::from_str("3.12").unwrap();
//!
//! // Target directory to install Python to.
//! let target = PathBuf::from("...");
//!
//! // Use selection strategy to resolve for the best matching release available.
//! let strategy = Strategy::Selection(Options { version: Some(version), kind: "cpython", os: "apple", architecture: "aarch64", build_configuration: "pgo+lto"});
//!
//! let release = resolve_release(&strategy).unwrap();
//!
//! install_with_target(&release, target).unwrap();
//! ```

pub use crate::install::install_with_target;
pub use crate::resolve::{resolve_release, Options, RequestedVersion, Strategy};

mod install;
mod releases;
mod resolve;
