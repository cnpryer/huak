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
    /// Initialize `Project` from a given path. If a manifest isn't found
    /// at the path, then we search for a manifest and set the project root
    /// if it's found.
    pub fn from(path: PathBuf) -> Result<Project, anyhow::Error> {
        let config = Config::from(&path)?;
        let venv = Venv::find(&path)?;
        let manifest_path = &config.manifest().path;

        // Set the root to the directory the manifest file was found.
        // TODO: This is probably not the right way to do this.
        let mut root = path;
        if let Some(parent) = manifest_path.parent() {
            root = parent.to_path_buf()
        }

        Ok(Project { root, config, venv })
    }
}

impl PythonProject for Project {
    /// Get a reference to the `Project` `Config`.
    fn config(&self) -> &Config {
        &self.config
    }

    /// Get a reference to the `Project` `Venv`.
    // TODO: Decouple to operate on `Config` data.
    fn venv(&self) -> &Option<Venv> {
        &self.venv
    }

    /// Set the `Project`'s `Venv`.
    // TODO: Decouple to operate on `Config` data.
    fn set_venv(&mut self, venv: Venv) {
        self.venv = Some(venv);
    }
}
