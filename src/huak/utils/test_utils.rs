use std::{
    env,
    path::{Path, PathBuf},
};

use fs_extra::dir;

use crate::{
    env::venv::Venv,
    project::{python::PythonProject, Project},
};

pub fn get_resource_dir() -> PathBuf {
    let cwd = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(cwd).join("resources")
}

pub fn create_venv(path: PathBuf) -> Result<Venv, anyhow::Error> {
    let venv = Venv::new(path);
    venv.create().unwrap();

    Ok(venv)
}

// Creates a mock `Project` from a `path`. A mock `Project` is given a
// re-usable .venv from cwd
pub fn create_mock_project(path: PathBuf) -> Result<Project, anyhow::Error> {
    let cwd = env::current_dir()?;
    let venv = create_venv(cwd.join(".venv"))?;

    let mut mock_project = Project::from(path)?;
    mock_project.set_venv(venv);

    Ok(mock_project)
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
