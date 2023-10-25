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
//! ```
//! huak_python_manager install 3.12 --target <path-to-target-dir>
//! ```
//!
//! ### Rust
//!
//! #### Install a Python release to a target directory with minimal configuration.
//!
//! TODO(cnpryer): Design better API.
//!
//! ```rust
//! use std::path::PathBuf;
//! use huak_python_manager::{Options, Strategy, install_with_target};
//!
//!
//! // The version of the Python to install.
//! let version = RequestedVersion::from_str("3.12").unwrap();
//!
//! // Target directory to install Python to.
//! let target = PathBuf::from("...");
//!
//! // Use selection strategy to resolve for the best matching release available.
//! let strategy = Strategy::Selection(Options { version, kind: "cpython", os: "apple", architecture: "aarch64", build_configuration: "pgo+lto"});
//!
//! install_with_target(strategy, target).unwrap();
//!
//! ```

pub use crate::install::install_with_target;
pub use crate::resolve::{Options, RequestedVersion, Strategy};

mod install;
mod releases;
mod resolve;
