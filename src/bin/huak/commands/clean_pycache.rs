use super::utils::subcommand;
use anyhow::Error;
use clap::Command;
use glob::{glob, Paths, PatternError};
use huak::errors::{CliError, CliErrorType, CliResult};
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

pub fn cmd() -> Command<'static> {
    subcommand("clean-pycache")
        .about("Remove all .pyc files and __pycache__ directories.")
}

pub fn run() -> CliResult<()> {
    let mut success: bool = true;

    let mut _error: Option<Error> = None;
    for i in get_delete_patterns() {
        let files: Result<Paths, PatternError> = glob(&i.glob);

        success = success
            && match files {
                Ok(paths) => {
                    let mut file_level_success = true;
                    for path in paths {
                        match path {
                            Ok(p) => match i.path_type {
                                PathType::Directory => {
                                    match remove_dir_all(p) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            file_level_success = false;
                                            _error = Some(Error::new(e));
                                        }
                                    }
                                }
                                PathType::File => match remove_file(p) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        file_level_success = false;
                                        _error = Some(Error::new(e));
                                    }
                                },
                            },
                            Err(e) => {
                                file_level_success = false;
                                _error = Some(Error::new(e))
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
        Err(CliError::new(CliErrorType::IOError))
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
