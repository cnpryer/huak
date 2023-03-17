use crate::error::HuakResult;
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
}
