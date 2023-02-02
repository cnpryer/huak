use std::{env, path::PathBuf};

use tempfile::tempdir;

use crate::{env::venv::Venv, errors::HuakResult, project::Project};

use super::path::copy_dir;

pub fn get_resource_dir() -> PathBuf {
    let cwd = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(cwd).join("resources")
}

pub fn create_venv(path: PathBuf) -> HuakResult<Venv> {
    let venv = Venv::new(path);

    venv.create()?;

    Ok(venv)
}

// Creates a mock `Project` from a `path`. A mock `Project` is given a
// re-usable .venv from cwd
pub fn create_mock_project(path: PathBuf) -> HuakResult<Project> {
    let cwd = env::current_dir()?;
    create_venv(cwd.join(".venv"))?;

    Ok((Project::from_directory(path)).unwrap())
}

/// Creates a mock `Project`, copying it from "mock_project" directory
pub fn create_mock_project_full() -> HuakResult<Project> {
    let directory = tempdir().unwrap().into_path();
    let mock_project_path = get_resource_dir().join("mock-project");
    copy_dir(&mock_project_path, &directory);

    create_mock_project(directory.join("mock-project"))
}
