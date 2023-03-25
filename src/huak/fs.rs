use crate::error::{Error, HuakResult};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Copy contents from one directory into a new directory at a provided `to` full path.
/// If the `to` directory doesn't exist this function creates it.
pub fn copy_dir<T: AsRef<Path>>(from: T, to: T) -> HuakResult<()> {
    let (from, to) = (from.as_ref(), to.as_ref());
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from));
    let target_root = to.to_path_buf();
    let from_component_count = from.to_path_buf().components().count();
    while let Some(working_path) = stack.pop() {
        // Collects the trailing components of the path
        let src: PathBuf = working_path
            .components()
            .skip(from_component_count)
            .collect();
        let dest = if src.components().count() == 0 {
            target_root.clone()
        } else {
            target_root.join(&src)
        };
        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }
        for entry in fs::read_dir(working_path)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name() {
                fs::copy(&path, dest.join(filename))?;
            }
        }
    }

    Ok(())
}

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
///   2. Search all sub-directories one level for `file_name`.
///   3. If `file_name` is found, return its path.
///   4. Else step one level up from its parent's path and decrement the
///      recursion limit.
pub fn find_file_bottom_up(
    file_name: &str,
    from: impl AsRef<Path>,
    recursion_limit: usize,
) -> HuakResult<Option<PathBuf>> {
    let from = from.as_ref();
    if !from.exists() || recursion_limit == 0 {
        return Ok(None);
    }
    if from.join(file_name).exists() {
        return Ok(Some(from.join(file_name)));
    }
    // Search all sub-directory roots for target_file
    if let Some(path) = fs::read_dir(from)?
        .into_iter()
        .filter(|item| item.is_ok())
        .map(|item| item.expect("failed to map dir entry").path()) // TODO: Is there better than .expect?
        .filter(|item| item.is_dir())
        .find(|item| item.join(file_name).exists())
    {
        return Ok(Some(path.join(file_name)));
    };
    // If nothing is found from searching the subdirectories then perform the same search from
    // the parent directory.
    return find_file_bottom_up(
        file_name,
        from.parent().ok_or(Error::InternalError(
            "failed to establish a parent directory".to_string(),
        ))?,
        recursion_limit - 1,
    );
}

pub fn last_path_component(path: impl AsRef<Path>) -> HuakResult<String> {
    let path = path.as_ref();
    match path
        .components()
        .last()
        .ok_or(Error::InternalError(format!(
            "failed to parse project name from {}",
            path.display()
        )))?
        .as_os_str()
        .to_str()
    {
        Some(it) => Ok(it.to_string()),
        None => {
            return Err(Error::InternalError(format!(
                "failed to parse project name from {}",
                path.display()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_copy_dir() {
        let to = tempdir().unwrap().into_path();
        let from = crate::test_resources_dir_path().join("mock-project");
        copy_dir(from, to.join("mock-project")).unwrap();

        assert!(to.join("mock-project").exists());
        assert!(to.join("mock-project").join("pyproject.toml").exists());
    }

    #[test]
    fn test_find_file_bottom_up() {
        let tmp = tempdir().unwrap().into_path();
        let from = crate::test_resources_dir_path().join("mock-project");
        copy_dir(&from, &tmp.join("mock-project")).unwrap();
        let res =
            find_file_bottom_up("pyproject.toml", &tmp.join("mock-project"), 5);

        assert!(res.unwrap().unwrap().exists());
    }
}
