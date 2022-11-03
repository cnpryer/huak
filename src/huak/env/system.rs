use crate::errors::{HuakError, HuakResult};
use std::env;
use std::fs;

/// Gets the PATH environment variable and splits this on ':'.
fn parse_path() -> HuakResult<Vec<String>> {
    let path_str = match env::var("PATH") {
        Ok(path) => path,
        Err(e) => return Err(HuakError::EnvVarError(e)),
    };

    Ok(path_str.split(':').map(|dir| dir.to_string()).collect())
}

/// Takes a binary name and searches the entire dir, if it finds the binary it will return the path
/// to the binary by appending the bin name to the dir.
///
/// returns on the first hit
fn find_binary(bin_name: String, dir: &str) -> HuakResult<Option<String>> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(e) => return Err(HuakError::IOError(e)),
    };

    for bin in read_dir {
        if let Ok(dir_entry) = bin {
            if let Some(file_name) = dir_entry.file_name().to_str() {
                if file_name == bin_name {
                    return Ok(Some(format!("{}/{}", dir, bin_name)));
                }
            }
        }
    }
    Ok(None)
}

/// Returns the full path of the python binary in a sepcific order. Python 2 is depcreated so
/// python3 is prefered. If there is no python3 some distributions also rename python3 to simply
/// python. See [PEP394])https://peps.python.org/pep-0394/)
pub fn find_python_binary_paths() -> HuakResult<String> {
    let paths = parse_path()?;

    for path in paths {
        if let Ok(optional) = find_binary("python3".to_string(), &path) {
            if let Some(python) = optional {
                return Ok(python);
            }
        }
        if let Ok(optional) = find_binary("python".to_string(), &path) {
            if let Some(python) = optional {
                return Ok(python);
            }
        }
        if let Ok(optional) = find_binary("python3".to_string(), &path) {
            if let Some(python) = optional {
                return Ok(python);
            }
        }
    }

    Err(HuakError::PythonNotFound)
}

#[cfg(test)]
mod tests {
    use super::find_binary;

    #[test]
    fn find_binary_test() {
        let binary_name: String = "test_bashrc".to_string();

        let dir: String = "test_files".to_string();

        let correct_output = "test_files/test_bashrc".to_string();

        let found_bin = find_binary(binary_name, &dir).unwrap().unwrap();

        assert_eq!(correct_output, found_bin);
    }
}
