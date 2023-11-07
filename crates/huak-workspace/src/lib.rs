//! # Workspace
//!
//! This crate implements `Workspace`. `resolve_root` can be used to resolve a root directory of a workspace.
//!
//! ```rust
//! use huak_workspace::{PathMarker, resolve_root};
//! use tempfile::TempDir;
//!
//! // dir (root)
//! // └── project (member)
//! // | └── pyproject.toml
//! // └── pyproject.toml
//! let dir = TempDir::new().unwrap();
//! let cwd = dir.path().join("package");
//! let ws = resolve_root(cwd, PathMarker::file("pyproject.toml"));
//!
//! assert!(ws.root().exists());
//! assert_eq!(ws.root(), dir.path());
//!```
pub use resolve::{resolve_first, resolve_root, PathMarker};
use std::path::{Path, PathBuf};

mod resolve;

/// A workspace is a directory on a file system. Workspaces can consist of members -- which can
/// be wokspaces of their own.
///
/// Given some current working directory, a workspace can be resolved by finding the root of
/// the workspace.
///
/// ```rust
/// use huak_workspace::Workspace;
/// use std::path::PathBuf;
///
/// let cwd = PathBuf::new();
/// let ws = Workspace::new(cwd);
/// ```
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
    members: Option<Vec<Self>>,
}

impl Workspace {
    pub fn new<T: AsRef<Path>>(root: T) -> Self {
        new_workspace(root)
    }

    #[must_use]
    pub fn members(&self) -> Option<&Vec<Self>> {
        self.members.as_ref()
    }
}

impl Workspace {
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

// Root paths should be directories.
fn new_workspace<T: AsRef<Path>>(path: T) -> Workspace {
    if path.as_ref().is_dir() {
        return Workspace {
            root: path.as_ref().to_path_buf(),
            members: None,
        };
    }

    if let Some(parent) = path.as_ref().parent() {
        if parent.is_dir() {
            return Workspace {
                root: parent.to_path_buf(),
                members: None,
            };
        }
    }

    Workspace {
        root: path.as_ref().to_path_buf(),
        members: None,
    }
}
