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
    pub path: Option<PathBuf>,
    spec: Option<String>,
}

impl LocalTool {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        // TODO(cnpryer): More robust
        Self::from(path.into())
    }

    #[must_use]
    pub fn spec(&self) -> Option<&String> {
        self.spec.as_ref()
    }

    #[must_use]
    pub fn from_spec(name: String, spec: String) -> Self {
        Self {
            name,
            path: None,
            spec: Some(spec),
        }
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.as_ref().map_or(false, |it| it.exists())
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

impl From<PathBuf> for LocalTool {
    fn from(value: PathBuf) -> Self {
        LocalTool {
            name: name_from_path(&value)
                .map(ToString::to_string)
                .unwrap_or_default(),
            path: Some(value.clone()),
            spec: None,
        }
    }
}
