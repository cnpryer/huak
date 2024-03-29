use std::{env, path::PathBuf};

/// Huak's home directory is located at ~/.huak.
///
/// # Unix
///
/// On unix systems the `HOME` environment variable is used if it exists.
///
/// # Windows
///
/// On windows the `USERPROFILE` environment variable is used if it exists.
#[must_use]
pub fn huak_home_dir() -> Option<PathBuf> {
    env::var("HUAK_HOME")
        .ok()
        .map(PathBuf::from)
        .or(sys::home_dir().map(|p| p.join(".huak")))
}

pub mod sys {
    use super::PathBuf;

    #[cfg(windows)]
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("USERPROFILE").map(PathBuf::from).ok()
    }

    #[cfg(any(unix, target_os = "redox"))]
    #[must_use]
    pub fn home_dir() -> Option<PathBuf> {
        #[allow(deprecated)]
        std::env::home_dir()
    }
}
