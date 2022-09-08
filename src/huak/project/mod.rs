pub mod python;
use std::path::PathBuf;

use crate::{config::Config, env::venv::Venv};

use self::python::PythonProject;

#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    config: Config,
    venv: Option<Venv>,
}

impl Project {
    /// Initializes `Project` from a `root` path. This function attempts to generate a `Config`
    /// by scanning the root of the project for configuration files such as pyproject.toml.
    /// If a venv is found at the root of the project it will also initalize a `Venv`. A venv
    /// is expected to be either .venv or venv at the root.
    pub fn new(root: PathBuf) -> Project {
        let config = Config::new(root.as_path()).unwrap_or(Config::default());
        let venv = Venv::find(root.as_path());
        let venv = if venv.is_err() {
            None
        } else {
            Some(venv.unwrap())
        };

        Project { root, config, venv }
    }
}

impl PythonProject for Project {
    fn config(&self) -> &Config {
        &self.config
    }

    fn venv(&self) -> &Option<Venv> {
        &self.venv
    }
}
