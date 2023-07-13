use crate::error::{Error, HuakResult};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
pub fn copy_dir<T: AsRef<Path>>(
    from: T,
    to: T,
    options: &CopyDirOptions,
) -> Result<(), Error> {
    let from = from.as_ref();
    let to = to.as_ref();

    if from.is_dir() {
        for entry in fs::read_dir(from)?.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if options.exclude.contains(&entry_path) {
                continue;
            }

            let destination = to.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                fs::create_dir_all(&destination)?;
                copy_dir(entry.path(), destination, options)?;
            } else {
                fs::copy(entry.path(), &destination)?;
            }
        }
    }
    Ok(())
}

#[derive(Default)]
pub struct CopyDirOptions {
    /// Exclude paths
    pub exclude: Vec<PathBuf>,
}

/// Get an iterator over all paths found in each directory.
pub fn flatten_directories(
    directories: impl IntoIterator<Item = PathBuf>,
) -> impl Iterator<Item = PathBuf> {
    directories
        .into_iter()
        .filter_map(|p| p.read_dir().ok())
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
}

/// Search for the path to a target file from a given directory's path and the file_name.
/// The search is executed with the following steps:
///   1. Get all sub-directories.
///   2. Search all sub-directory roots for file_name.
///   3. If file_name is found, return its path.
///   4. Else step one directory up until the `last` directory has been searched.
pub fn find_root_file_bottom_up<T: AsRef<Path>>(
    file_name: &str,
    dir: T,
    last: T,
) -> HuakResult<Option<PathBuf>> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Ok(None);
    }
    if dir.join(file_name).exists() {
        return Ok(Some(dir.join(file_name)));
    }
    // Search all sub-directory roots for target_file.
    if let Some(path) = fs::read_dir(dir)?
        .filter(|item| item.is_ok())
        .map(|item| item.expect("failed to map dir entry").path())
        .filter(|item| item.is_dir())
        .find(|item| item.join(file_name).exists())
    {
        return Ok(Some(path.join(file_name)));
    };
    if dir == last.as_ref() {
        return Ok(None);
    }
    // If nothing is found from searching the subdirectories then perform the same search from
    // the parent directory.
    find_root_file_bottom_up(
        file_name,
        dir.parent().ok_or(Error::InternalError(
            "failed to establish a parent directory".to_string(),
        ))?,
        last.as_ref(),
    )
}

/// Get the last component of a path. For example this function would return
/// "dir" from the following path:
/// /some/path/to/some/dir
pub fn last_path_component<T: AsRef<Path>>(path: T) -> HuakResult<String> {
    let path = path.as_ref();
    let path = path
        .components()
        .last()
        .ok_or(Error::InternalError(format!(
            "failed to parse path {}",
            path.display()
        )))?
        .as_os_str()
        .to_str()
        .ok_or(Error::InternalError(format!(
            "failed to parse path {}",
            path.display()
        )))?
        .to_string();
    Ok(path)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_copy_dir() {
        let to = tempdir().unwrap().into_path();
        let from = crate::test_resources_dir_path().join("mock-project");
        copy_dir(from, to.join("mock-project"), &CopyDirOptions::default())
            .unwrap();

        assert!(to.join("mock-project").exists());
        assert!(to.join("mock-project").join("pyproject.toml").exists());
    }

    #[test]
    fn test_find_root_file_bottom_up() {
        let tmp = tempdir().unwrap().into_path();
        let from = crate::test_resources_dir_path().join("mock-project");
        copy_dir(&from, &tmp.join("mock-project"), &CopyDirOptions::default())
            .unwrap();
        let res = find_root_file_bottom_up(
            "pyproject.toml",
            tmp.join("mock-project").as_path(),
            tmp.as_path(),
        );

        assert!(res.unwrap().unwrap().exists());
    }
}
