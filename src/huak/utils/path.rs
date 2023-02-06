use std::{
    fs,
    path::{Path, PathBuf},
};

use fs_extra::dir;

use crate::errors::{HuakError, HuakResult};

/// Return the filename from a `Path`.
pub fn parse_filename(path: &Path) -> HuakResult<&str> {
    // Attempt to convert OsStr to str.
    let name = match path.file_name() {
        Some(f) => f.to_str(),
        _ => {
            return Err(HuakError::InternalError(
                "failed to read name from path".into(),
            ))
        }
    };

    // If a str was failed to be parsed error.
    if name.is_none() {
        return Err(HuakError::InternalError(format!(
            "failed to convert filename from {} to string",
            path.display()
        )));
    }

    Ok(name.unwrap())
}

/// Convert a `Path` to a &str.
pub fn to_string(path: &Path) -> HuakResult<&str> {
    let res = match path.to_str() {
        Some(it) => it,
        None => {
            return Err(HuakError::InternalError(format!(
                "failed to convert {} to a string",
                path.display()
            )))
        }
    };

    Ok(res)
}

/// Copies one directory into another.
pub fn copy_dir(from: &Path, to: &Path) {
    if !from.exists() {
        eprintln!("resource archive does not exist");
    }

    if !to.exists() {
        eprintln!("`to` {} does not exist", to.display());
    }

    // Copy mock project dir to target dir
    let copy_options = dir::CopyOptions::new();
    dir::copy(from, to, &copy_options).unwrap();
}

// Notes about the Python Environment work:
// - I don’t think there are any built-in (standardization) ways to determine when a package was installed. You could probably look at certain file metadata, but I’m not sure how reliable that is.
// - Cache dir should be managed by the installer.
/// NOTE: This is a special function that walks directories upward and only one level
/// deep.
/// TODO: Maybe rename.
/// Search for the path to a target file from a given directory's path and the filename.
/// The search is executed with the following steps:
///   1. Get all sub-directories.
///   2. Search all sub-directories one level for `filename`.
///   3. If `filename` is found, return its path.
///   4. Else step one level up from its parent's path and decrement the
///      recursion limit.
pub fn search_directories_for_file(
    from: &Path,
    filename: &str,
    recursion_limit: usize,
) -> HuakResult<Option<PathBuf>> {
    if !from.exists() || recursion_limit == 0 {
        return Ok(None);
    }

    if from.join(filename).exists() {
        return Ok(Some(from.join(filename)));
    }

    // Search all sub-directories one step. Exclude any directories that were already searched.
    let subdirectories: Vec<PathBuf> = fs::read_dir(from)?
        .into_iter()
        .filter(|it| it.is_ok())
        .map(|it| it.expect("failed to map dir entry").path()) // TODO: Is there better than .expect?
        .filter(|it| it.is_dir())
        .collect();

    // TODO: This is not efficient.
    for dir in subdirectories.iter() {
        if dir.join(filename).exists() {
            return Ok(Some(dir.join(filename)));
        }
    }

    // If nothing is found from searching the subdirectories then perform the same search from
    // the parent directory.
    return search_directories_for_file(
        from.parent().ok_or(HuakError::PyVenvNotFoundError)?,
        filename,
        recursion_limit - 1,
    );
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::test_utils::get_resource_dir;

    #[test]
    fn test_copy_dir() {
        let tmp = tempdir().unwrap().into_path();
        let from = get_resource_dir().join("mock-project");

        copy_dir(&from, &tmp);

        assert!(tmp.join("mock-project").exists());
        assert!(tmp.join("mock-project").join("pyproject.toml").exists());
    }

    #[test]
    fn test_search_parents_for_filepath() {
        let tmp = tempdir().unwrap().into_path();
        let from = get_resource_dir().join("mock-project");

        copy_dir(&from, &tmp);

        let res = search_directories_for_file(
            &tmp.join("mock-project"),
            "pyproject.toml",
            5,
        );

        assert!(res.unwrap().unwrap().exists());
    }
}
