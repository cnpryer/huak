use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};
use glob::{glob, Paths, PatternError};
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

/// Clean build artifacts from a `Project`.
pub fn clean_project(project: &Project) -> HuakResult<()> {
    // Just find dist at project root.
    let dist_path = project.root().join("dist");

    // If it's there delete it, otherwise just return Ok.
    if !dist_path.is_dir() {
        return Ok(());
    }

    Ok(remove_dir_all(dist_path)?)
}

// TODO: From project root.
pub fn clean_project_pycache() -> HuakResult<()> {
    for i in get_delete_patterns() {
        let files: Result<Paths, PatternError> = glob(&i.glob);

        match files {
            Ok(paths) => {
                for path in paths {
                    match path {
                        Ok(p) => match i.path_type {
                            PathType::Directory => {
                                remove_dir_all(p).map_err(|e| {
                                    HuakError::InternalError(e.to_string())
                                })?;
                            }
                            PathType::File => {
                                remove_file(p).map_err(|e| {
                                    HuakError::InternalError(e.to_string())
                                })?;
                            }
                        },
                        Err(e) => {
                            return Err(HuakError::InternalError(
                                e.to_string(),
                            ));
                        }
                    }
                }
            }
            Err(e) => return Err(HuakError::InternalError(e.to_string())),
        };
    }
    Ok(())
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

// TODO: Test clean_project_pycache
#[cfg(test)]
mod tests {
    use super::*;

    use std::{env, path::PathBuf};

    use tempfile::tempdir;

    use crate::utils::{path::copy_dir, test_utils::create_mock_project};

    #[test]
    pub fn clean() {
        let directory = tempdir().unwrap().into_path();
        let from_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("mock-project");

        copy_dir(&from_dir, &directory);

        let project =
            create_mock_project(directory.join("mock-project")).unwrap();
        let had_dist = project.root().join("dist").exists();

        clean_project(&project).unwrap();

        assert!(had_dist);
        assert!(project.root().as_path().exists());
        assert!(project.root().as_path().join("pyproject.toml").exists());
        assert!(project
            .root()
            .as_path()
            .join("src")
            .join("mock_project")
            .join("__init__.pyc")
            .exists());
    }
}
