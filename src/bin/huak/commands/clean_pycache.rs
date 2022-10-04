use crate::errors::{CliError, CliResult};

use glob::{glob, Paths, PatternError};
use huak::errors::HuakError;
use std::{
    fs::{remove_dir_all, remove_file},
    process::ExitCode,
};

#[derive(Clone, Copy)]
enum PathType {
    Directory,
    File,
}
struct DeletePath {
    path_type: PathType,
    glob: String,
}

pub fn run() -> CliResult<()> {
    let mut success: bool = true;

    let mut _error: Option<HuakError> = None;
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
                                            _error = Some(e.into());
                                        }
                                    }
                                }
                                PathType::File => match remove_file(p) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        file_level_success = false;
                                        _error = Some(e.into());
                                    }
                                },
                            },
                            Err(e) => {
                                file_level_success = false;
                                _error = Some(HuakError::InternalError(
                                    e.to_string(),
                                ));
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
        Err(CliError::new(
            _error.unwrap_or_else(|| {
                HuakError::UnknownError("An unknown error ocurred".into())
            }),
            ExitCode::FAILURE,
        ))
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
