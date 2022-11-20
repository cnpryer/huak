use std::env;
use std::fs;
use std::path::PathBuf;

use crate::errors::{HuakError, HuakResult};

// TODO
const RECURSION_LIMIT: usize = 5;

const PYTHON_BINARY_TARGETS: [PythonBinary; 3] = [
    PythonBinary::Python3,
    PythonBinary::Python,
    PythonBinary::Python2,
];

enum PythonBinary {
    Python,
    Python3,
    Python2,
}

impl PythonBinary {
    fn as_str(&self) -> &'static str {
        match &self {
            PythonBinary::Python => "python",
            PythonBinary::Python3 => "python3",
            PythonBinary::Python2 => "python2",
        }
    }
}

/// Returns the full path of the python binary in a sepcific order. Python 2 is depcreated so
/// python3 is prefered. If there is no python3 some distributions also rename python3 to simply
/// python. See [PEP394](https://peps.python.org/pep-0394/)
/// TODO: Refactor to evaluate against each file during the search.
pub fn find_python_binary_path(
    from_dir: Option<PathBuf>,
) -> HuakResult<String> {
    let paths = match from_dir {
        Some(path) => vec![path],
        None => parse_path_var()?,
    };

    for path in paths {
        for target in PYTHON_BINARY_TARGETS.iter() {
            if let Ok(Some(python)) = find_binary(target.as_str(), &path, 0) {
                return Ok(python);
            }
        }
    }

    Err(HuakError::PythonNotFound)
}

/// Gets the PATH environment variable and splits this on ':'.
fn parse_path_var() -> HuakResult<Vec<PathBuf>> {
    let path_str = match env::var("PATH") {
        Ok(path) => path,
        Err(e) => return Err(HuakError::EnvVarError(e)),
    };

    Ok(path_str.split(':').map(|dir| dir.into()).collect())
}

/// Takes a binary name and searches the entire dir, if it finds the binary it will return the path
/// to the binary by appending the bin name to the dir.
///
/// returns on the first hit
fn find_binary(
    bin_name: &str,
    dir: &PathBuf,
    step: usize,
) -> HuakResult<Option<String>> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(e) => return Err(HuakError::IOError(e)),
    };

    // TODO
    if step > RECURSION_LIMIT {
        return Ok(None);
    }

    for dir_entry in read_dir.flatten() {
        if let Ok(file_type) = dir_entry.file_type() {
            if file_type.is_dir() {
                if let Ok(bin_path) =
                    find_binary(bin_name, &dir_entry.path(), step + 1)
                {
                    return Ok(bin_path);
                }
            } else if let Some(file_name) = dir_entry.file_name().to_str() {
                if file_name == bin_name {
                    if let Some(bin_path) = dir_entry.path().to_str() {
                        return Ok(Some(bin_path.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_python_search_windows() -> Result<(), std::io::Error> {
        let directory = tempdir().unwrap();

        fs::write(directory.path().join("python.exe"), "")?;

        let expected_python =
            String::from(directory.path().join("python.exe").to_str().unwrap());

        assert_eq!(
            find_binary("python.exe", &directory.into_path(), 0).unwrap(),
            Some(expected_python)
        );

        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_python_search_macos() -> Result<(), std::io::Error> {
        let directory = tempdir().unwrap();

        fs::write(directory.path().join("python"), "")?;

        let expected_python =
            String::from(directory.path().join("python").to_str().unwrap());

        assert_eq!(
            find_binary("python", &directory.into_path(), 0).unwrap(),
            Some(expected_python)
        );

        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_python_search_linux() -> Result<(), std::io::Error> {
        let directory = tempdir().unwrap();

        fs::write(directory.path().join("python3"), "")?;

        let expected_python =
            String::from(directory.path().join("python3").to_str().unwrap());

        assert_eq!(
            find_binary("python3", &directory.into_path(), 0).unwrap(),
            Some(expected_python)
        );

        Ok(())
    }
}
