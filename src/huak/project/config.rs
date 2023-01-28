use std::path::{Path, PathBuf};

use crate::config::pyproject::toml::Toml;

use crate::errors::HuakResult;
use crate::package::PythonPackage;
use crate::utils;

// TODO: Env/programmatically.
const DEFAULT_SEARCH_STEPS: usize = 5;

/// Traits for Python-specific configuration.
pub trait PythonConfig {
    fn package_list(&self) -> Vec<PythonPackage>;
    fn optional_package_list(&self, group: &str) -> Vec<PythonPackage>;
}

/// Project configuration contains data from different Python project
/// configuration files including a pyproject toml.
#[derive(Default)]
pub struct ProjectConfig {
    pyproject_path: PathBuf,
    pyproject_toml: Toml,
}

// TODO: Config refactor.
impl ProjectConfig {
    pub fn from(path: &Path) -> HuakResult<ProjectConfig> {
        let manifest_path = utils::path::search_parents_for_filepath(
            path,
            "pyproject.toml",
            DEFAULT_SEARCH_STEPS,
        )?;

        let manifest_path = match manifest_path {
            Some(it) => it,
            None => return Ok(ProjectConfig::default()), // Just use the toml for now.
        };
        let pyproject_toml = Toml::open(&manifest_path)?;

        Ok(ProjectConfig {
            pyproject_path: manifest_path,
            pyproject_toml,
        })
    }

    /// Get a reference to the project name from manifest data.
    // TODO: Use more than toml.
    pub fn project_name(&self) -> &String {
        let table = &self.pyproject_toml.project;

        &table.name
    }

    pub fn set_project_name(&mut self, name: &str) {
        self.pyproject_toml.project.name = name.to_string();
    }

    /// Get a reference to the project version from manifest data.
    // TODO: Use more than toml.
    pub fn project_version(&self) -> &Option<String> {
        let table = &self.pyproject_toml.project;

        &table.version
    }

    pub fn pyproject_path(&self) -> &PathBuf {
        &self.pyproject_path
    }

    pub fn pyproject_toml(&self) -> &Toml {
        &self.pyproject_toml
    }
}

impl PythonConfig for ProjectConfig {
    // Get vec of `PythonPackage`s from the manifest.
    // TODO: More than toml.
    fn package_list(&self) -> Vec<PythonPackage> {
        // Get huak's spanned table found in the Toml.
        let table = &self.pyproject_toml.project;
        let empty: Vec<String> = Vec::new();
        // Dependencies to list from.
        let from = &table.dependencies.as_ref().unwrap_or(&empty);

        // Collect into vector of owned `PythonPackage` data.
        from.iter()
            .filter_map(|d| PythonPackage::from_str(d).ok())
            .collect()
    }
    // Get vec of `PythonPackage`s from the manifest.
    // TODO: More than toml.
    fn optional_package_list(&self, opt_group: &str) -> Vec<PythonPackage> {
        // Get huak's spanned table found in the Toml.
        let table = &self.pyproject_toml.project;
        let empty: Vec<String> = vec![];

        // Dependencies to list from.
        let from = &table
            .optional_dependencies
            .as_ref()
            .map_or(&empty, |deps| deps.get(opt_group).unwrap_or(&empty));

        from.iter()
            .filter_map(|d| PythonPackage::from_str(d).ok())
            .collect()
    }
}
