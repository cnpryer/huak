pub mod config;
pub mod python;
use std::path::PathBuf;

use crate::env::venv::Venv;

use self::config::Config;
use self::python::PythonProject;

#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    config: Config,
    venv: Option<Venv>,
}

impl Project {
    // Initialize `Project` from a given path. If a manifest isn't found
    // at the path, then we search for a manifest and set the project root
    // if it's found.
    pub fn from(path: PathBuf) -> Result<Project, anyhow::Error> {
        let config = Config::from(&path)?;
        let venv = Venv::find(&path)?;
        let manifest_path = &config.manifest.path;
        let mut root = path;
        if let Some(parent) = manifest_path.parent() {
            root = parent.to_path_buf()
        }

        Ok(Project { root, config, venv })
    }
}

impl PythonProject for Project {
    fn config(&self) -> &Config {
        &self.config
    }

    fn venv(&self) -> &Option<Venv> {
        &self.venv
    }

    fn set_venv(&mut self, venv: Venv) {
        self.venv = Some(venv);
    }
}
