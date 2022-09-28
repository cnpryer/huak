use std::{env, path::PathBuf};

use crate::{env::venv::Venv, errors::HuakError, project::Project};

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
pub fn create_mock_project(path: PathBuf) -> Result<Project, HuakError> {
    let cwd = match env::current_dir() {
        Ok(p) => p,
        Err(e) => return Err(HuakError::AnyHowError(anyhow::format_err!(e))),
    };
    let venv = create_venv(cwd.join(".venv"))?;

    let mut mock_project = Project::from(path)?;
    mock_project.set_venv(venv);

    Ok(mock_project)
}
