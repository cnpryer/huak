use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{Channel, LocalToolchain};

#[derive(Default)]
pub struct LocalToolchainResolver;

impl LocalToolchainResolver {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn from_path<T: AsRef<Path>>(self, channel: &Channel, path: T) -> Option<LocalToolchain> {
        if path_matches_name(channel, &path) {
            Some(LocalToolchain::new(path.as_ref()))
        } else {
            None
        }
    }

    pub fn from_dir<T: AsRef<Path>>(self, channel: &Channel, path: T) -> Option<LocalToolchain> {
        resolve_from_dir(channel, path)
    }

    #[must_use]
    pub fn from_paths(&self, channel: &Channel, paths: &[PathBuf]) -> Option<LocalToolchain> {
        paths
            .iter()
            .find(|p| path_matches_name(channel, p))
            .map(LocalToolchain::new)
    }
}

pub enum Entry {
    String(String),
}

fn resolve_from_dir<T: AsRef<Path>>(channel: &Channel, path: T) -> Option<LocalToolchain> {
    let Ok(paths) = fs::read_dir(path.as_ref()) else {
        return None;
    };

    // Return the first matching toolchain.
    for entry in paths.flatten() {
        let p = entry.path();
        if path_matches_name(channel, &p) {
            return Some(LocalToolchain::new(p));
        }
    }

    None
}

fn path_matches_name<T: AsRef<Path>>(channel: &Channel, path: T) -> bool {
    match channel {
        Channel::Default => path_name_matches(path, "default"),
        Channel::Descriptor(descriptor) => path_name_matches(path, &descriptor.to_string()),
        Channel::Version(version) => path_name_matches(path, &version.to_string()),
    }
}

fn path_name_matches<T>(path: T, name: &str) -> bool
where
    T: AsRef<Path>,
{
    path.as_ref()
        .file_name()
        .map_or(false, |it| it.eq_ignore_ascii_case(name))
}
