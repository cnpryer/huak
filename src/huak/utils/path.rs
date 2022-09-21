use std::path::{Path, PathBuf};

use anyhow::{self, Error};
use fs_extra::dir;

/// Return the filename from a `Path`.
pub fn parse_filename(path: &Path) -> Result<&str, Error> {
    // Attempt to convert OsStr to str.
    let name = match path.file_name() {
        Some(f) => f.to_str(),
        _ => return Err(anyhow::format_err!("failed to read name from path")),
    };

    // If a str was failed to be parsed error.
    if name.is_none() {
        return Err(anyhow::format_err!(
            "failed to convert filename from {} to string",
            path.display()
        ));
    }

    Ok(name.unwrap())
}

/// Convert a `Path` to a &str.
pub fn to_string(path: &Path) -> Result<&str, anyhow::Error> {
    let pip_path = match path.to_str() {
        Some(s) => s,
        None => {
            return Err(anyhow::format_err!(
                "failed to convert {} to a string",
                path.display()
            ))
        }
    };

    Ok(pip_path)
}

/// Search for manifest files using a path `from` to start from and
/// `steps` to recurse.
pub fn search_parents_for_filepath(
    from: &Path,
    filename: &str,
    steps: usize,
) -> Result<Option<PathBuf>, anyhow::Error> {
    if steps == 0 {
        return Ok(None);
    }

    if from.join(filename).exists() {
        return Ok(Some(from.join(filename)));
    }

    if let Some(parent) = from.parent() {
        return search_parents_for_filepath(parent, filename, steps - 1);
    }

    Ok(None)
}

/// Copies one directory into another.
pub fn copy_dir(from: &PathBuf, to: &PathBuf) -> bool {
    if !Path::new(from).is_file() {
        eprintln!("resource archive does not exist");
    }

    if !Path::new(to).is_dir() {
        eprintln!("`to` {} does not exist", to.display());
    }

    // Copy mock project dir to target dir
    let copy_options = dir::CopyOptions::new();
    dir::copy(from.as_path(), to.as_path(), &copy_options).unwrap();

    true
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::test_utils::get_resource_dir;

    #[test]
    fn test_copy_dir() {
        let tmp = tempdir().unwrap().into_path().to_path_buf();
        let from = get_resource_dir().join("mock-project");

        copy_dir(&from, &tmp);

        assert!(tmp.join("mock-project").exists());
        assert!(tmp.join("mock-project").join("pyproject.toml").exists());
    }

    #[test]
    fn test_search_parents_for_filepath() {
        let tmp = tempdir().unwrap().into_path().to_path_buf();
        let from = get_resource_dir().join("mock-project");

        copy_dir(&from, &tmp);

        let res = search_parents_for_filepath(
            &tmp.join("mock-project").join("src"),
            "pyproject.toml",
            5,
        );

        assert!(res.unwrap().unwrap().exists());
    }
}
