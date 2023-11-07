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

pub(crate) struct PathResolver {
    pub cwd: PathBuf,
    pub marker: PathMarker,
    pub strategy: ResolveStrategy,
}

impl PathResolver {
    pub(crate) fn resolve(&self) -> Workspace {
        match self.strategy {
            ResolveStrategy::ResolveRoot => best_root(&self.cwd, &self.marker),
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
