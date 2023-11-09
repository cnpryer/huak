use crate::name_from_path;
use std::{fmt::Display, path::PathBuf, str::FromStr};

/// The local tool for Huak's toolchain system.
///
/// A `LocalTool` provides a small wrapper for tool paths.
/// ```rust
/// use std::path::PathBuf;
/// use huak_toolchain::LocalTool;
///
/// let path = PathBuf::new();
/// let tool = LocalTool::new(&path);
///
/// assert_eq!(&path, &tool.path);
/// ```
#[derive(Clone, Debug)]
pub struct LocalTool {
    pub name: String,
    pub path: PathBuf,
}

impl LocalTool {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        // TODO(cnpryer): More robust
        let path = path.into();

        Self {
            name: name_from_path(&path)
                .map(ToString::to_string)
                .unwrap_or_default(),
            path,
        }
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

impl Display for LocalTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for LocalTool {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LocalTool::new(s))
    }
}
