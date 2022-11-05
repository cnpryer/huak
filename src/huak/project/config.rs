use std::path::{Path, PathBuf};

use crate::config::pyproject::toml::Toml;

use crate::errors::HuakResult;
use crate::package::python::PythonPackage;
use crate::utils;

// TODO: Env/programmatically.
const DEFAULT_SEARCH_STEPS: usize = 5;

/// Traits for Python-specific configuration.
pub trait PythonConfig {
    fn package_list(&self) -> Vec<PythonPackage>;
    fn optional_package_list(&self, opt_group: &str) -> Vec<PythonPackage>;
}

// TODO: PythonConfig?
#[derive(Default)]
pub struct Config {
    pub(crate) path: PathBuf,
    pub(crate) toml: Toml,
}

// TODO: Config refactor.
impl Config {
    pub fn from(path: &Path) -> HuakResult<Config> {
        let manifest_path = utils::path::search_parents_for_filepath(
            path,
            "pyproject.toml",
            DEFAULT_SEARCH_STEPS,
        )?;

        let manifest_path = match manifest_path {
            Some(it) => it,
            None => return Ok(Config::default()), // Just use the toml for now.
        };
        let toml = Toml::open(&manifest_path)?;

        Ok(Config {
            path: manifest_path,
            toml,
        })
    }

    /// Get a reference to the project name from manifest data.
    // TODO: Use more than toml.
    pub fn project_name(&self) -> &String {
        let table = &self.toml.project;

        &table.name
    }

    pub fn set_project_name(&mut self, name: &str) {
        self.toml.project.name = name.to_string();
    }

    /// Get a reference to the project version from manifest data.
    // TODO: Use more than toml.
    pub fn project_version(&self) -> &Option<String> {
        let table = &self.toml.project;

        &table.version
    }
}

impl PythonConfig for Config {
    // Get vec of `PythonPackage`s from the manifest.
    // TODO: More than toml.
    fn package_list(&self) -> Vec<PythonPackage> {
        // Get huak's spanned table found in the Toml.
        let table = &self.toml.project;
        let empty: Vec<String> = Vec::new();
        // Dependencies to list from.
        let from = &table.dependencies.as_ref().unwrap_or(&empty);

        // Collect into vector of owned `PythonPackage` data.
        from.iter()
            .filter_map(|d| PythonPackage::from(d).ok())
            .collect()
    }
    // Get vec of `PythonPackage`s from the manifest.
    // TODO: More than toml.
    fn optional_package_list(&self, opt_group: &str) -> Vec<PythonPackage> {
        // Get huak's spanned table found in the Toml.
        let table = &self.toml.project;
        let empty: Vec<String> = vec![];

        // Dependencies to list from.
        let from = &table
            .optional_dependencies
            .as_ref()
            .map_or(&empty, |deps| deps.get(opt_group).unwrap_or(&empty));

        from.iter()
            .filter_map(|d| PythonPackage::from(d).ok())
            .collect()
    }
}
