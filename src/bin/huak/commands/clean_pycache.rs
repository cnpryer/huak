use crate::utils::subcommand;
use anyhow::Error;
use clap::Command;
use glob::{glob, Paths, PatternError};
use huak::errors::{CliError, CliResult};

use std::fs::{remove_dir_all, remove_file};

#[derive(Clone, Copy)]
enum PathType {
    Directory,
    File,
}
struct DeletePath {
    path_type: PathType,
    glob: String,
}

pub fn arg() -> Command<'static> {
    subcommand("clean-pycache").about("Remove all .pyc files and __pycache__ directories.")
}

pub fn run() -> CliResult {
    let mut success: bool = true;

    let mut error: Option<Error> = None;
    for i in get_delete_patterns() {
        let files: Result<Paths, PatternError> = glob(&i.glob);

        success = success
            && match files {
                Ok(paths) => {
                    let mut file_level_success = true;
                    for path in paths {
                        match path {
                            Ok(p) => match i.path_type {
                                PathType::Directory => match remove_dir_all(p) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        file_level_success = false;
                                        error = Some(Error::new(e));
                                    }
                                },
                                PathType::File => match remove_file(p) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        file_level_success = false;
                                        error = Some(Error::new(e));
                                    }
                                },
                            },
                            Err(e) => {
                                file_level_success = false;
                                error = Some(Error::new(e))
                            }
                        }
                    }

                    file_level_success
                }

                // this should not happen as it would be a compile time issue
                _ => false,
            }
    }

    if success {
        Ok(())
    } else {
        Err(CliError {
            error,
            exit_code: 2,
        })
    }
}

fn get_delete_patterns() -> Vec<DeletePath> {
    vec![
        DeletePath {
            path_type: PathType::Directory,
            glob: "**/__pycache__".to_owned(),
        },
        DeletePath {
            path_type: PathType::File,
            glob: "**/*.pyc".to_owned(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::{env::set_current_dir, path::PathBuf};

    use super::*;
    use tempfile::tempdir;

    use crate::test_utils::create_py_project_sample;
    use glob::glob;

    #[test]
    pub fn assert_no_pyc() {
        let directory: PathBuf = tempdir().unwrap().into_path().to_path_buf();
        let from_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("mock-project");

        create_py_project_sample(&from_dir, &directory);
        set_current_dir(&directory).unwrap();

        let _ = run();
        let i = glob("**/*.pyc").unwrap().count();
        assert!(i == 0 as usize);
        assert!(directory.as_path().join("mock-project").exists());
        assert!(directory
            .as_path()
            .join("mock-project")
            .join("pyproject.toml")
            .exists());
        assert!(directory
            .as_path()
            .join("mock-project")
            .join("src")
            .join("mock_project")
            .exists());
        assert!(!directory
            .as_path()
            .join("mock-project")
            .join("src")
            .join("mock_project")
            .join("__init__.pyc")
            .exists());
    }
}
