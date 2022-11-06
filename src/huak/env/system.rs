use std::env;
use std::fs;
use std::path::PathBuf;

use crate::errors::{HuakError, HuakResult};

/// Returns the full path of the python binary in a sepcific order. Python 2 is depcreated so
/// python3 is prefered. If there is no python3 some distributions also rename python3 to simply
/// python. See [PEP394])https://peps.python.org/pep-0394/)
pub fn find_python_binary_path(
    from_dir: Option<PathBuf>,
) -> HuakResult<String> {
    let paths = match from_dir {
        Some(path) => vec![path],
        None => parse_path()?,
    };

    for path in paths {
        if let Ok(Some(python)) = find_binary("python3".to_string(), &path) {
            return Ok(python);
        }
        if let Ok(Some(python)) = find_binary("python".to_string(), &path) {
            return Ok(python);
        }
        if let Ok(Some(python)) = find_binary("python2".to_string(), &path) {
            return Ok(python);
        }
    }

    Err(HuakError::PythonNotFound)
}

/// Gets the PATH environment variable and splits this on ':'.
fn parse_path() -> HuakResult<Vec<PathBuf>> {
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
fn find_binary(bin_name: String, dir: &PathBuf) -> HuakResult<Option<String>> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(e) => return Err(HuakError::IOError(e)),
    };

    for dir_entry in read_dir.flatten() {
        if let Some(file_name) = dir_entry.file_name().to_str() {
            if file_name == bin_name {
                return Ok(Some(format!("{}/{}", dir.display(), bin_name)));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn find_custom_binary() {
        let binary_name: String = "test_bashrc".to_string();

        let dir: PathBuf = "test_files".into();

        let correct_output = "test_files/test_bashrc".to_string();

        let found_bin = find_binary(binary_name, &dir).unwrap().unwrap();

        assert_eq!(correct_output, found_bin);
    }

    #[test]
    fn find_correct_python() {
        let test_path: PathBuf = "test_files/bin".into();

        let python = find_python_binary_path(Some(test_path)).unwrap();

        assert_eq!("test_files/bin/python3".to_string(), python);
    }
}
