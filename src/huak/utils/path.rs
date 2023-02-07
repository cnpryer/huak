use std::{
    fs,
    path::{Path, PathBuf},
};

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

/// Copy contents from one directory into a new directory at a provided `to` full path.
/// If the `to` directory doesn't exist this function creates it.
pub fn copy_dir(from: &Path, to: &Path) -> HuakResult<()> {
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

        copy_dir(&from, &tmp.join("mock-project")).unwrap();

        assert!(tmp.join("mock-project").exists());
        assert!(tmp.join("mock-project").join("pyproject.toml").exists());
    }

    #[test]
    fn test_search_parents_for_filepath() {
        let tmp = tempdir().unwrap().into_path();
        let from = get_resource_dir().join("mock-project");

        copy_dir(&from, &tmp.join("mock-project")).unwrap();

        let res = search_directories_for_file(
            &tmp.join("mock-project"),
            "pyproject.toml",
            5,
        );

        assert!(res.unwrap().unwrap().exists());
    }
}
