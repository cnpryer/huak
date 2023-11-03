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
//! use huak_python_manager::{ReleaseOptions, Strategy, install_with_target, resolve_release};
//!
//! // Target directory to install Python to.
//! let target = PathBuf::from("...");
//!
//! // Use selection strategy to resolve for the best matching release available.
//! let strategy = Strategy::Selection(ReleaseOptions::default());
//!
//! let release = resolve_release(&strategy).unwrap();
//!
//! install_with_target(&release, target).unwrap();
//! ```

pub use crate::error::Error;
pub use crate::resolve::{
    release_options_from_requested_version, resolve_release, ReleaseArchitecture,
    ReleaseBuildConfiguration, ReleaseKind, ReleaseOption, ReleaseOptions, ReleaseOs,
    RequestedVersion, Strategy,
};
pub use crate::version::Version;
use install::download_release;
pub use install::install_with_target;
pub use releases::Release;
use std::path::Path;
use tar::Archive;
use zstd::stream::read::Decoder;

mod error;
mod install;
mod releases;
mod resolve;
mod version;

// A simple API for managing Python installs.
pub struct PythonManager;

impl Default for PythonManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonManager {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn download(&self, release: Release<'static>) -> Result<Vec<u8>, Error> {
        download_release(&release)
    }

    pub fn unpack<T: AsRef<Path>>(&self, bytes: &[u8], to: T, decode: bool) -> Result<(), Error> {
        if decode {
            let decoded =
                Decoder::with_buffer(bytes).map_err(|e| Error::ZstdError(e.to_string()))?;
            let mut archive = Archive::new(decoded);

            // TODO(cnpryer): Support more archive formats.
            archive
                .unpack(to)
                .map_err(|e| Error::TarError(e.to_string()))
        } else {
            todo!()
        }
    }
}
