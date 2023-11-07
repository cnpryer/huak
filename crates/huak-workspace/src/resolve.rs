use crate::Workspace;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

/// Resolve a workspace by searching for a root.
///
/// ```no_run
/// use huak_workspace::{PathMarker, resolve_root};
/// use std::path::PathBuf;
///
/// let cwd = PathBuf::from("root/member/");
/// let marker = PathMarker::file("pyproject.toml");
/// let ws = resolve_root(&cwd, marker);
/// ```
pub fn resolve_root<T: Into<PathBuf>>(cwd: T, marker: PathMarker) -> Workspace {
    let resolver = PathResolver {
        cwd: cwd.into(),
        marker,
        strategy: ResolveStrategy::ResolveRoot,
    };

    resolver.resolve()
}

/// Resolve a workspace by searching for a root.
///
/// ```no_run
/// use huak_workspace::{PathMarker, resolve_first};
/// use std::path::PathBuf;
///
/// let cwd = PathBuf::from("root/member/");
/// let marker = PathMarker::file("pyproject.toml");
/// let ws = resolve_first(&cwd, marker);
/// ```
pub fn resolve_first<T: Into<PathBuf>>(cwd: T, marker: PathMarker) -> Workspace {
    let resolver = PathResolver {
        cwd: cwd.into(),
        marker,
        strategy: ResolveStrategy::ResolveFirst,
    };

    resolver.resolve()
}

pub(crate) struct PathResolver {
    pub cwd: PathBuf,
    pub marker: PathMarker,
    pub strategy: ResolveStrategy,
}

impl PathResolver {
    pub(crate) fn resolve(&self) -> Workspace {
        match self.strategy {
            ResolveStrategy::ResolveRoot => best_root(&self.cwd, &self.marker),
            ResolveStrategy::ResolveFirst => first_root(&self.cwd, &self.marker),
        }
    }
}

fn best_root<T: AsRef<Path>>(cwd: T, marker: &PathMarker) -> Workspace {
    let mut root = dir(cwd.as_ref());

    for p in cwd.as_ref().ancestors() {
        if has_marker(p, marker) {
            // Roots should be directories
            root = dir(p);
        }
    }

    let members = resolve_members(root.as_path(), marker);

    Workspace { root, members }
}

fn first_root<T: AsRef<Path>>(cwd: T, marker: &PathMarker) -> Workspace {
    let mut ws = Workspace::new(cwd.as_ref());

    for p in cwd.as_ref().ancestors() {
        if has_marker(p, marker) {
            ws = Workspace::new(dir(p));
            break;
        }
    }

    ws.members = resolve_members(ws.root(), marker);

    ws
}

fn dir<T: AsRef<Path>>(path: T) -> PathBuf {
    let path = path.as_ref();

    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(std::path::Path::to_path_buf)
            .expect("path buff")
    }
}

fn resolve_members<T: AsRef<Path>>(path: T, marker: &PathMarker) -> Option<Vec<Workspace>> {
    let Ok(paths) = std::fs::read_dir(path) else {
        return None;
    };

    let mut members = Vec::new();

    for entry in paths.flatten() {
        let p = entry.path();

        if has_marker(&p, marker) {
            members.push(Workspace::new(&p));
        }
    }

    if members.is_empty() {
        None
    } else {
        Some(members)
    }
}

fn has_marker<T: AsRef<Path>>(path: T, marker: &PathMarker) -> bool {
    let path = path.as_ref();

    match marker {
        PathMarker::File(name) if path.is_dir() => path.join(name).exists(),
        PathMarker::Dir(name) | PathMarker::File(name) => matches_file_name(path, name),
    }
}

fn matches_file_name<T: AsRef<Path>>(path: T, name: &str) -> bool {
    path.as_ref()
        .file_name()
        .map_or(false, |s| s.eq_ignore_ascii_case(name))
}

#[derive(Default)]
pub(crate) enum ResolveStrategy {
    // Traverse from some location a first steps forward and a few steps backwards.
    #[default]
    ResolveRoot,
    ResolveFirst,
}

#[derive(Debug)]
pub enum PathMarker {
    File(String),
    Dir(String),
}

impl PathMarker {
    #[must_use]
    pub fn file(s: &str) -> Self {
        Self::File(s.to_string())
    }

    #[must_use]
    pub fn dir(s: &str) -> Self {
        Self::Dir(s.to_string())
    }
}

impl Display for PathMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathMarker::File(name) | PathMarker::Dir(name) => write!(f, "{name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::{resolve_root, PathMarker};
    use std::{fs::File, io::Write};
    use tempfile::TempDir;

    #[test]
    fn test_resolve_root() {
        let dir = TempDir::new().unwrap();
        let mock = create_mock_ws(dir.as_ref());
        let cwd = mock.join("package");
        let ws = resolve_root(cwd, PathMarker::file("pyproject.toml"));

        assert!(ws.root().exists());
        assert_eq!(ws.root(), dir.path());
    }

    #[test]
    fn test_first_root() {
        let dir = TempDir::new().unwrap();
        let mock = create_mock_ws(dir.as_ref());
        let cwd = mock.join("package");
        let ws = resolve_first(&cwd, PathMarker::file("pyproject.toml"));

        assert_eq!(&cwd, ws.root());
    }

    // Create a mock workspace and return its path.
    fn create_mock_ws(path: &Path) -> PathBuf {
        let sub = path.join("package");

        std::fs::create_dir_all(&sub).unwrap();

        let marker = "pyproject.toml";

        let mut file = File::create(path.join(marker)).unwrap();
        file.write_all(&[]).unwrap();

        let mut file = File::create(sub.join(marker)).unwrap();
        file.write_all(&[]).unwrap();

        path.to_path_buf()
    }
}
