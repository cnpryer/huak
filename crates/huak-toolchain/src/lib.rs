//! # The toolchain implementation for Huak.
//!
//! ## Toolchain
//!
//! - Channel
//! - Path
//! - Tools
//!
//! ## Channels
//!
//! Channels are used to identify toolchains.
//!
//! - major.minor of a Python interpreter
//! - major.minor.patch of a Python interpreter
//! - Complete Python interpreter identifying chains (for example, 'cpython-3.12.0-apple-aarch64-pgo+lto')
//! - Etc.
//!
//! ## Path
//!
//! A unique toolchain is identifiable by the path it's installed to. A directory contains the entire toolchain.
//!
//! ## Tools
//!
//! Toolchains are composed of installed tools. The default tools installed are:
//!
//! - python (and Python installation management system)
//! - ruff
//! - mypy (TODO(cnpryer): May be replaced)
//! - pytest (TODO(cnpryer): May be replaced)
//!
//! ## Other
//!
//! Tools are centralized around a common Python inerpreter installed to the toolchain. The toolchain utilizes
//! a virtual environment shared by the tools in the toolchain. A bin directory contains the symlinked tools.
//! If a platform doesn't support symlinks hardlinks are used.
//!
//! ## `huak-toolchain`
//!
//! This crate implements Huak's toolchain via `Channel`, `Toolchain`, and `Tool`.
//!
//! ### `LocalToolchain`
//!
//! A directory containing `LocalTool`s for Huak to use.
//!
//! ### `LocalTool`
//!
//! A local tool that Huak can use. A `Tool` in a `Toolchain` is considered to have a `name` and a `path`.
//!
//! Local tools can be executable programs.
//!
//! ```rust
//! use huak_toolchain::LocalToolchain;
//! use std::path::PathBuf;
//!
//! let path = PathBuf::from("path/to/toolchain/");
//! let toolchain = LocalToolchain::new(&path);
//! let py = toolchain.tool("python");
//! let bin = path.join("bin");
//! let py_bin = bin.join("python");
//!
//! assert_eq!(&py.name, "python");
//! assert_eq!(py.path, py_bin);
//! ```
//!
//! Use `toolchain.try_with_proxy_tool(tool)` to attempt to create a proxy file installed to the toolchain.
//! Use `toolchain.try_with_tool(tool)` to perform the full copy of the tool.
//!
//! The bin of the toolchain directory is intended to be added to users' scopes. So the bin directory
//! may contain full copies of executable programs or proxies to them.
//!
//! ```
//!
//! ```
//! export PATH="/path/to/toolchain/bin/:$PATH"
//! ```

pub use channel::{Channel, DescriptorParts};
pub use error::Error;
use path::name_from_path;
pub use resolve::LocalToolchainResolver;
pub use settings::SettingsDb;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::{
    fs::{self, hard_link, read_link},
    path::{Path, PathBuf},
};
pub use tools::LocalTool;

mod channel;
mod error;
mod path;
mod resolve;
mod settings;
mod tools;

#[derive(Debug)]
pub struct LocalToolchain {
    inner: LocalToolchainInner,
}

/// The local toolchain for Huak.
///
/// A local toolchain is created for different channels. The channel determines its
/// release installs and its path.
///
/// A `LocalToolchain` is meant to be used as an API for toolchain management on some
/// filesystem.
///
/// ```rust
/// use std::path::PathBuf;
/// use huak_toolchain::LocalToolchain;
///
///
/// let root = PathBuf::new();
/// let toolchain = LocalToolchain::new(&root);
///
/// assert_eq!(toolchain.root(), &root);
/// assert_eq!(toolchain.bin(), root.join("bin"));
/// ```
impl LocalToolchain {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        let path = path.into();

        Self {
            inner: LocalToolchainInner {
                name: name_from_path(&path)
                    .ok()
                    .map_or(String::from("default"), ToString::to_string),
                channel: Channel::Default,
                path,
            },
        }
    }

    pub fn set_channel(&mut self, channel: Channel) -> &mut Self {
        self.inner.channel = channel;
        self
    }

    #[must_use]
    pub fn name(&self) -> &String {
        &self.inner.name
    }

    #[must_use]
    pub fn tool(&self, name: &str) -> LocalTool {
        LocalTool::new(self.bin().join(name))
    }

    #[must_use]
    pub fn channel(&self) -> &Channel {
        &self.inner.channel
    }

    #[must_use]
    pub fn bin(&self) -> PathBuf {
        self.root().join("bin")
    }

    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.inner.path
    }

    #[must_use]
    pub fn tools(&self) -> Vec<LocalTool> {
        let mut tools = Vec::new();

        if let Ok(entries) = fs::read_dir(self.bin()) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    let p = entry.path();

                    if p == self.bin().join(file_name) {
                        tools.push(LocalTool {
                            name: file_name.to_string(),
                            path: self.bin().join(file_name),
                        });
                    }
                }
            }
        }

        tools
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.root().exists()
    }

    #[must_use]
    pub fn tool_is_installed(&self, name: &str) -> bool {
        self.bin().join(name).exists()
    }

    #[must_use]
    pub fn downloads(&self) -> PathBuf {
        self.root().join("downloads")
    }

    #[must_use]
    pub fn info(&self) -> String {
        let tools = self.tools();

        format!(
            "Name: {}\nTools: {}\nPath: {}",
            self.name(),
            tools
                .into_iter()
                .map(|it| it.name)
                .collect::<Vec<_>>()
                .join(", "),
            self.bin().display()
        )
    }

    #[must_use]
    pub fn with_channel(self, channel: Channel) -> Self {
        Self {
            inner: LocalToolchainInner {
                name: self.inner.name,
                channel,
                path: self.inner.path,
            },
        }
    }

    /// Register a local tool from its path. `allow_copy` as `false` would restrict setup
    /// as a light proxy to the tool. With `allow_copy` as `true` the setup can resort to
    /// heavier tool installs including full copies.
    // TODO(cnpryer): hard_link vs full_copy
    pub fn register_tool<T: AsRef<Path>>(
        &self,
        path: T,
        name: &str,
        allow_copy: bool,
    ) -> Result<(), Error> {
        let original = path.as_ref();
        let link = self.bin().join(name);
        let link = link.as_ref();

        // If we can read the link we'll just make our own symlink of the original link's linked file
        let source = if original.is_symlink() {
            read_link(original)?
        } else {
            original.to_path_buf()
        };

        let path = if source.is_absolute() {
            source
        } else if let Some(parent) = original.parent() {
            parent.join(source)
        } else {
            std::fs::canonicalize(source)?
        };

        // Try to symlink. Then try hardlink. Then try fs::copy.
        let path = path.as_path();

        let Err(symlink_err) = try_symlink(path, link) else {
            return Ok(());
        };

        if allow_copy {
            hard_link(path, link)?;
            // TODO(cnpryer): Hardlink or look into it more
            // If copy is allowed then try hardlink then resort to full copy.
            // let _copied = std::fs::copy(path, link)?;
            Ok(())
        } else {
            Err(symlink_err)
        }
    }
}

#[derive(Debug)]
pub struct LocalToolchainInner {
    name: String,
    channel: Channel,
    path: PathBuf,
}

impl From<PathBuf> for LocalToolchain {
    fn from(value: PathBuf) -> Self {
        LocalToolchain::new(value)
    }
}

fn try_symlink<T: AsRef<Path>>(original: T, link: T) -> Result<(), Error> {
    #[cfg(unix)]
    let err = symlink(original, link);

    #[cfg(windows)]
    let err = symlink_file(original, link);

    Ok(err?)
}
